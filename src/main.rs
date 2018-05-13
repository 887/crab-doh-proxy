#![deny(deprecated)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
//#![allow(unreachable_code)]

extern crate futures;

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
extern crate tokio_tls;
extern crate tokio_service;

extern crate hyper;
extern crate hyper_tls;

mod dns;
mod server;
mod request;

use std::sync::Arc;

use std::io::prelude::*;
use std::io::{self, Write};
use std::env;
use std::net::{SocketAddr, UdpSocket};

use tokio::prelude::*;
use tokio::net::UdpSocket as TokioUdpSocket;
use tokio::net::TcpStream;
use tokio::reactor::Handle;
use tokio::runtime::Runtime;

use rayon::{ThreadPool, ThreadPoolBuilder};

use log::{SetLoggerError, LevelFilter};

use server::Server;

fn get_sockets(
    adr: SocketAddr,
    reactor_handle: &tokio::reactor::Handle,
) -> std::io::Result<(UdpSocket, TokioUdpSocket)> {
    let socket = std::net::UdpSocket::bind(adr)?;
    let socket_for_later = socket.try_clone()?;
    info!("Listening on: {}", socket.local_addr().unwrap());
    let tokio_socket = TokioUdpSocket::from_std(socket, reactor_handle)?;
    Ok((socket_for_later, tokio_socket))
}

fn spawn_server (addr: &str, runtime: &mut Runtime, boxed_pool: Arc<ThreadPool>) -> std::io::Result<()> {
    let addr = addr.parse().unwrap();
    let (udp_socket, tokio_socket) = get_sockets(addr, runtime.reactor())?;
    let server = Server {
        threadpool: boxed_pool,
        tokio_socket: tokio_socket,
        socket: udp_socket,
        buf: vec![0; 1500],
    };

    let server = server.map_err(|e| println!("server error = {:?}", e));
    runtime.spawn(server);

    Ok(())
}

fn main() -> std::io::Result<()> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let mut runtime = Runtime::new().unwrap();

    //0 causes the build to either use the cpu count or the RAYON_NUM_THREADS environment variable
    let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(0)
                .build()
                .unwrap();
    let pool = Arc::new(pool);

    //TODO: read a https over dns target from clap, cloudflare as default and google as backup
    //
    //cloudlfare, also needs the content-type header for application/dns-json
    //"1.1.1.1" ,1.0.0.1, dns.cloudflare.com
    // -> /dns-query?{name}{type}
    //
    //or the good old google
    //216.58.195.78, dns.google.com
    // -> /resolve?{name}{type}
    //
    // step 2 is to check the certificate and certificate authority

    //TODO: parse list of listening addresses via clap and spawn a server for each like this
    spawn_server("127.0.0.1:6142", &mut runtime, pool.clone())?;

    runtime.shutdown_on_idle().wait().unwrap();

    Ok(())
}
