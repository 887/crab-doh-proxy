#![deny(deprecated)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
//#![allow(unreachable_code)]

#[macro_use]
extern crate tokio;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate rayon;

extern crate dns_parser;

extern crate simple_logger;
#[macro_use]
extern crate log;
extern crate env_logger;

extern crate native_tls;

mod dns;
mod request;
mod resolver;
mod config;
#[cfg(feature = "mock_request")]
mod mock_request;
#[cfg(not(feature = "mock_request"))]
mod server;

use std::sync::Arc;

use std::io::prelude::*;
use std::io::{self, Write};
use std::env;
use std::net::{SocketAddr, UdpSocket};

use tokio::prelude::*;
use tokio::net::TcpStream;
use tokio::reactor::Handle;
use tokio::runtime::Runtime;

use rayon::{ThreadPool, ThreadPoolBuilder};

use log::{SetLoggerError, LevelFilter};

#[cfg(not(feature = "mock_request"))]
use server::{Server, spawn_server};
use config::Config;

#[cfg(feature = "mock_request")]
fn main() -> std::io::Result<()> {
    mock_request::mock_request()
}

#[cfg(not(feature = "mock_request"))]
fn main() -> std::io::Result<()> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let mut runtime = Runtime::new().unwrap();

    //TODO: make num-worker-threads an option read from the command line arguments via clap
    //0 causes the build to either use the cpu count or the RAYON_NUM_THREADS environment variable
    let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(0)
                .build()
                .unwrap();
    let pool = Arc::new(pool);

    //TODO: read a doh resolver target from clap, cloudflare as default and google as backup
    //Only let build-in resolvers be selectable, we don't need the mess that is  generic support
    //for everything via user configuration.

    let config = Config::init_cloudflare();
    // let config = Config::init_google();
    let config = Arc::new(config);

    //TODO: parse list of listening addresses via clap and spawn a server for each like this
    spawn_server("127.0.0.1:6142", &mut runtime, pool.clone(), config.clone())?;

    runtime.shutdown_on_idle().wait().unwrap();

    Ok(())
}
