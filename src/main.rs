#![deny(deprecated)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

#[macro_use]
extern crate tokio;

extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

extern crate rayon;

use std::io::prelude::*;
use std::{env};
use std::net::{SocketAddr, UdpSocket};
use std::io::{self, Write};

use tokio::net::UdpSocket as TokioUdpSocket;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::reactor::Handle;
use tokio::runtime::Runtime;

use rayon::{ThreadPool, ThreadPoolBuilder};

mod dns;

struct Server{
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
                Async::Ready((size, peer)) => {
                    handle_packet(self, size, peer)?;
                },
                Async::NotReady => { },
            }
        }
    }
}

fn handle_packet(server: &mut Server, size: usize, peer: SocketAddr) -> Result<(), io::Error> {
    println!("Received {} bytes from {}", size, peer);

    //TODO: make tcp request to doh-server, parse reult, reply in new thread on the server socket.
    //clone it again with try_clone if necessary for lifetime requirements!

    //TODO: remove this:
    let amt = server.tokio_socket.poll_send_to(&server.buf[..size], &peer)?;
    println!("Echoed {:?}/{} bytes to {}", amt, size, peer);

    Ok(())
}

fn get_sockets(adr: SocketAddr, reactor_handle: &tokio::reactor::Handle) -> std::io::Result<(UdpSocket, TokioUdpSocket)> {
    let socket = std::net::UdpSocket::bind(adr)?;
    let socket_for_later = socket.try_clone()?;
    println!("Listening on: {}", socket.local_addr().unwrap());
    let tokio_socket = TokioUdpSocket::from_std(socket, reactor_handle)?;
    Ok((socket_for_later, tokio_socket))
}

fn main() -> std::io::Result<()> {
    #![allow(unreachable_code)]

    let mut runtime = Runtime::new().unwrap();

    let pool = rayon::ThreadPoolBuilder::new().num_threads(0).build().unwrap();

    let addr = "127.0.0.1:6142".parse().unwrap();
    let (udp_socket,tokio_socket) = get_sockets(addr, runtime.reactor())?;
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
