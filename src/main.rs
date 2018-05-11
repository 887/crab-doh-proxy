#![deny(deprecated)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

#[macro_use]
extern crate tokio;

use tokio::io;
use tokio::prelude::*;
use tokio::net::UdpSocket;
use std::net::SocketAddr;

use std::{env};

struct Server {
    socket: UdpSocket,
    buf: Vec<u8>,
}

// fn main() {
//     use tokio::net::TcpListener;
//     let addr = "127.0.0.1:6142".parse().unwrap();
//     let listener = TcpListener::bind(&addr).unwrap();
//
//     let server = listener.incoming().for_each(|socket| {
//         println!("accepted socket; addr={:?}", socket.peer_addr().unwrap());
//
//         // Process socket here.
//
//         Ok(())
//     })
//     .map_err(|err| {
//         // All tasks must have an `Error` type of `()`. This forces error
//         // handling and helps avoid silencing failures.
//         //
//         // In our example, we are only going to log the error to STDOUT.
//         println!("accept error = {:?}", err);
//     });
//
//     println!("Hello, world!");
// }

impl Future for Server {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        loop {
            // First we check to see if there's a message we need to echo back.
            // If so then we try to send it back to the original source, waiting
            // until it's writable and we're able to do so.
            // if let Async::Ready((size, peer)) = self.to_send {
            //     let amt = self.socket.poll_send_to(&self.buf[..size], &peer).unwrap();
            //     // println!("Echoed {}/{} bytes to {}", amt, size, peer);
            //     self.to_send = Async::NotReady;
            // }

            // If we're here then `to_send` is `None`, so we take a look for the
            // next message we're going to echo back.
            match self.socket.poll_recv_from(&mut self.buf)? {
                Async::Ready((size, peer)) => {
                    println!("Received {} bytes from {}", size, peer);
                },
                Async::NotReady => { },
            }
        }
    }
}


fn main() {
    use tokio::net::UdpSocket;

    let addr = "127.0.0.1:6142".parse().unwrap();

    //udp sockets should be rebindable, right?
    let socket = UdpSocket::bind(&addr).unwrap();
    let socket_cloned = UdpSocket::bind(&addr).unwrap();


    println!("Listening on: {}", socket.local_addr().unwrap());

    let server = Server {
        socket: socket,
        buf: vec![0; 1024],
    };


    // This starts the server task.
    //
    // `map_err` handles the error by logging it and maps the future to a type
    // that can be spawned.
    //
    // `tokio::run` spawns the task on the Tokio runtime and starts running.
    tokio::run(server.map_err(|e| println!("server error = {:?}", e)));

}

