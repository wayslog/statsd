use bytes::{BufMut, BytesMut};
use crossbeam::{self, sync::MsQueue};
use futures::{Async, Stream};
use net2::UdpBuilder;
use tokio;
use tokio::net::UdpSocket;
use tokio::reactor::Handle;
use tokio::timer::Interval;

use com::*;

use std::convert::From;
use std::net;
use std::sync::Arc;
use std::time::{Duration, Instant};

const MAX_UDP_PACKET: usize = 2048;

/// collect will collect all statsd line by given metrcis
pub fn collect(queue: Arc<MsQueue<Metrics>>) -> Result<()> {
    let thread = CONFIG.thread();
    crossbeam::scope(|scope| {
        let jhs: Vec<_> = (0..thread)
            .into_iter()
            .map(|_| {
                let queue = queue.clone();
                scope.spawn(|| {
                    run_collector(&CONFIG.bind, Duration::from_millis(100), queue).unwrap()
                })
            })
            .collect();
        scope.defer(|| {
            let _: Vec<_> = jhs.into_iter().map(|jh| jh.join()).collect();
        });
    });
    Ok(())
}

pub struct BufferInterval<T, S, I>
where
    T: Stream<Item = I, Error = Error>,
    S: Store<Item = I>,
    I: Clone,
{
    interval: Interval,
    rx: T,
    store: S,
}

impl<T, S, I> BufferInterval<T, S, I>
where
    T: Stream<Item = I, Error = Error>,
    S: Store<Item = I>,
    I: Clone,
{
    fn new(rx: T, store: S, dur: Duration) -> Self {
        BufferInterval {
            interval: Interval::new(Instant::now(), dur),
            rx: rx,
            store: store,
        }
    }
}

impl<T, S, I> Stream for BufferInterval<T, S, I>
where
    T: Stream<Item = I, Error = Error>,
    S: Store<Item = I>,
    I: Clone,
{
    type Item = S;
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>> {
        loop {
            match self.interval.poll()? {
                Async::Ready(Some(_inst)) => {
                    let store = self.store.trancate();
                    return Ok(Async::Ready(Some(store)));
                }
                Async::Ready(None) => {
                    unreachable!();
                }
                Async::NotReady => {
                    trace!("can't trancate");
                }
            };

            match self.rx.poll()? {
                Async::Ready(Some(val)) => self.store.store(val),
                Async::NotReady => return Ok(Async::NotReady),
                Async::Ready(None) => return Ok(Async::Ready(None)),
            };
        }
    }
}

fn run_collector(bind: &str, dur: Duration, tx: Arc<MsQueue<Metrics>>) -> Result<()> {
    let socket = UdpSocket::from_std(reuseport_udp(bind)?, &Handle::current())?;
    let reader = LineReader {
        sock: socket,
        buf: BytesMut::with_capacity(MAX_UDP_PACKET),
    };

    let mes = Metrics {};
    let buf = BufferInterval::new(reader, mes, dur);
    let amt =
        buf.map_err(|x| {
            info!("get error with {:?}", x);
        }).for_each(move |store| {
            tx.push(store);
            Ok(())
        });

    tokio::run(amt);
    Ok(())
}

pub fn reuseport_udp(bind: &str) -> Result<net::UdpSocket> {
    use net2::unix::UnixUdpBuilderExt;
    let socket = UdpBuilder::new_v4()?
        .reuse_address(true)?
        .reuse_port(true)?
        .bind(bind)?;
    Ok(socket)
}

pub struct LineReader {
    sock: UdpSocket,
    buf: BytesMut,
}

impl LineReader {
    fn next_line(&mut self) -> Option<String> {
        if self.buf.is_empty() {
            return None;
        }

        if let Some(pos) = self.buf.iter().position(|&x| x == '\n' as u8) {
            let mut line = self.buf.split_to(pos + 1);
            let len = line.len();
            return String::from_utf8(line.split_to(len).to_vec()).ok();
        }
        String::from_utf8(self.buf.take().to_vec()).ok()
    }
}

impl Stream for LineReader {
    type Item = String;
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>> {
        loop {
            if let Some(line) = self.next_line() {
                trace!("new line read in buf {}", line);
                return Ok(Async::Ready(Some(line)));
            }
            self.buf.clear();

            let size = unsafe {
                let size = try_ready!(self.sock.poll_recv(self.buf.bytes_mut()));
                self.buf.advance_mut(size);
                trace!("read size {}", size);
                size
            };
            if size == 0 && self.buf.is_empty() {
                return Ok(Async::NotReady);
            }
        }
    }
}
