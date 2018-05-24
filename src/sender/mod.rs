use com::*;
use crossbeam::{self, sync::MsQueue};
use tokio::net::TcpStream;
use tokio::prelude::*;

use std::sync::Arc;

pub fn send(queue: Arc<MsQueue<Metrics>>) -> Result<()> {
    let worker = CONFIG.worker();
    crossbeam::scope(|scope| {
        let jhs: Vec<_> = (0..worker)
            .into_iter()
            .map(|_| {
                let queue = queue.clone();
                scope.spawn(|| run_sender(queue))
            })
            .collect();
        scope.defer(|| {
            let _: Vec<_> = jhs.into_iter().map(|jh| jh.join()).collect();
        });
    });
    Ok(())
}

pub fn dispatch(queue: Arc<MsQueue<Metrics>>) {}

fn run_sender(queue: Arc<MsQueue<Metrics>>) {}

pub struct GraphiteSender {
    sock: TcpStream,
}
