use std::sync::Arc;

use std::io::prelude::*;
use std::io::{self, Write};
use std::net::{UdpSocket};

use rayon::{ThreadPool};

use tokio::prelude::*;
use tokio::net::UdpSocket as TokioUdpSocket;

use request::handle_request;

pub struct Server {
    pub threadpool: Arc<ThreadPool>,
    pub socket: UdpSocket,
    pub tokio_socket: TokioUdpSocket,
    pub buf: Vec<u8>,
}

impl Future for Server {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        loop {
            match self.tokio_socket.poll_recv_from(&mut self.buf)? {
                Async::Ready((amt, scr_addr)) => {
                    handle_request(self, amt, scr_addr)?;
                }
                Async::NotReady => {}
            }
        }
    }
}

