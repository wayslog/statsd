#![feature(extern_prelude)]

extern crate bytes;
extern crate crc16;
extern crate crossbeam;
extern crate env_logger;
extern crate futures;
extern crate num_cpus;
extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate tokio_io;

#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

pub mod proxy;
pub mod ring;
pub use proxy::*;

pub mod com {
    use std::io;
    use std::num;
    use std::result;
    use std::str;

    pub use failure::Error;
    #[derive(Debug, Fail)]
    pub enum SError {
        #[fail(display = "placeholder")]
        Todo,
        #[fail(display = "error due to: {:?}", error)]
        IoError { error: io::Error },
        #[fail(display = "bad utf8 char: {:?}", error)]
        Utf8Error { error: str::Utf8Error },
        #[fail(display = "bad int str: {:?}", error)]
        ParseIntError { error: num::ParseIntError },
    }

    impl From<num::ParseIntError> for SError {
        fn from(oe: num::ParseIntError) -> SError {
            SError::ParseIntError { error: oe }
        }
    }

    impl From<str::Utf8Error> for SError {
        fn from(oe: str::Utf8Error) -> SError {
            SError::Utf8Error { error: oe }
        }
    }

    impl From<io::Error> for SError {
        fn from(oe: io::Error) -> SError {
            SError::IoError { error: oe }
        }
    }

    pub type Result<T> = result::Result<T, Error>;
}

pub use com::*;
// use com::{Result, Error};

pub fn worker() {
    init_log();
}

pub fn proxy() {
    init_log();
}

fn init_log() {
    env_logger::init();
    info!("statsd init log success");
}
