extern crate bytes;
extern crate crc16;
extern crate crossbeam;
extern crate env_logger;
#[macro_use]
extern crate futures;
extern crate net2;
extern crate num_cpus;
extern crate serde;
extern crate serde_json;
extern crate tokio;

#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

pub mod backends;
pub mod collector;
pub mod com;
pub mod sender;

use crossbeam::sync::MsQueue;

pub use com::*;
// use com::{Result, Error};
use std::sync::Arc;

pub fn worker() {
    init_log();
    let queue = Arc::new(MsQueue::new());
    collector::collect(queue.clone()).unwrap();
}

pub fn proxy() {
    init_log();
}

fn init_log() {
    env_logger::init();
    info!("statsd init log success");
}
