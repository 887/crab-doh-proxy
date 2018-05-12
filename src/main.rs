#![deny(deprecated)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

#[macro_use]
extern crate tokio;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate rayon;

extern crate dns_parser;

use std::io::prelude::*;
use std::env;
use std::net::{SocketAddr, UdpSocket};
use std::io::{self, Write};

use tokio::net::UdpSocket as TokioUdpSocket;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::reactor::Handle;
use tokio::runtime::Runtime;

use rayon::{ThreadPool, ThreadPoolBuilder};

use dns_parser::{Builder, Class, Packet, QueryClass, QueryType, ResponseCode, Type};

mod dns;

struct Server {
    threadpool: ThreadPool,
    socket: UdpSocket,
    tokio_socket: TokioUdpSocket,
    buf: Vec<u8>,
}

impl Future for Server {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        loop {
            match self.tokio_socket.poll_recv_from(&mut self.buf)? {
                Async::Ready((amt, scr_addr)) => {
                    handle_packet(self, amt, scr_addr)?;
                }
                Async::NotReady => {}
            }
        }
    }
}

struct RequestSource {
    socket: UdpSocket,
    src_addr: SocketAddr,
}

fn handle_packet(server: &mut Server, amt: usize, src_addr: SocketAddr) -> Result<(), io::Error> {
    println!("Received {} bytes from {}", amt, src_addr);

    //TODO: make tcp request to doh-server, parse reult, reply in new thread on the server socket.
    //clone it again with try_clone if necessary for lifetime requirements!

    let socket = server.socket.try_clone()?;
    let buf = server.buf.clone(); //clones this buffer
    server.threadpool.install(move || {
        parse_packet(socket, buf, src_addr);
    });

    Ok(())
}

fn parse_packet(socket: UdpSocket, buf: Vec<u8>, src_addr: SocketAddr) {
    let rs = RequestSource {socket: socket, src_addr: src_addr};
    send_response(rs, buf);
}

fn send_response(rs: RequestSource, buf: Vec<u8>) {
    let amt = buf.len();
    match rs.socket.send_to(&buf, &rs.src_addr) {
        Ok(_) => {
            println!("Echoed {:?}/{} bytes to {}", amt, amt, rs.src_addr);
        }
        Err(_) => {
            println!("Failed to send {:?}/{} bytes to {}", amt, amt, rs.src_addr);
        }
    };
}

fn get_sockets(
    adr: SocketAddr,
    reactor_handle: &tokio::reactor::Handle,
) -> std::io::Result<(UdpSocket, TokioUdpSocket)> {
    let socket = std::net::UdpSocket::bind(adr)?;
    let socket_for_later = socket.try_clone()?;
    println!("Listening on: {}", socket.local_addr().unwrap());
    let tokio_socket = TokioUdpSocket::from_std(socket, reactor_handle)?;
    Ok((socket_for_later, tokio_socket))
}

fn main() -> std::io::Result<()> {
    #![allow(unreachable_code)]

    let mut runtime = Runtime::new().unwrap();

    //0 causes the build to either use the cpu count or the RAYON_NUM_THREADS environment variable
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(0)
        .build()
        .unwrap();

    let addr = "127.0.0.1:6142".parse().unwrap();
    let (udp_socket, tokio_socket) = get_sockets(addr, runtime.reactor())?;
    let server = Server {
        threadpool: pool,
        tokio_socket: tokio_socket,
        socket: udp_socket,
        buf: vec![0; 1500],
    };

    let server = server.map_err(|e| println!("server error = {:?}", e));
    runtime.spawn(server);
    runtime.shutdown_on_idle().wait().unwrap();

    Ok(())
}
