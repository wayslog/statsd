use num_cpus;
use serde_json;
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

const DEFAULT_CONFIG_PATH: &str = "./test.cfg.json";

lazy_static! {
    pub static ref CONFIG: Config = {
        use std::env;
        use std::fs;
        let path = env::var("STATSD_CONFIG_FILE").unwrap_or(DEFAULT_CONFIG_PATH.to_string());
        let reader = fs::File::open(&path).expect("fail to open config file");
        serde_json::from_reader(reader).unwrap()
    };
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub bind: String,
    thread: Option<usize>,
    worker: Option<usize>,

    pub graphite: Option<GraphiteConfig>,
}

#[derive(Debug, Deserialize)]
pub struct GraphiteConfig {
    pub address: String,
    flush_interval: usize,
}

impl Config {
    pub fn worker(&self) -> usize {
        use std::usize;
        match self.worker {
            Some(ival) if ival == 0 => 1,
            Some(ival) => ival,
            None => {
                let num = num_cpus::get();
                let max_num = num / 3;
                usize::max(max_num, 1)
            }
        }
    }

    pub fn thread(&self) -> usize {
        match self.thread {
            Some(ival) if ival == 0 => 1,
            Some(ival) => ival,
            None => usize::max(num_cpus::get() * 2 / 3, 1),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Metrics {}

pub trait Store {
    type Item: Clone;
    fn store(&mut self, item: Self::Item);

    fn trancate(&mut self) -> Self;
}
