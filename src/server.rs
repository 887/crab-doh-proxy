use std::sync::Arc;

use std::io::prelude::*;
use std::io::{self, Read, Result, Write};
use std::io::Result as IoResult;
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::env;

use rayon::ThreadPool;

use tokio::prelude::*;
use tokio::net::UdpSocket as TokioUdpSocket;
use tokio::reactor::Handle;
use tokio::runtime::Runtime;

use config::Config;

use request::{parse_packet, RequestSource, WorkerResources};

pub struct Server {
    pub config: Arc<Config>,
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
                    handle_request(self, amt, scr_addr);
                }
                Async::NotReady => {}
            }
        }
    }
}

fn get_sockets(addr: SocketAddr, reactor_handle: &Handle) -> IoResult<(UdpSocket, TokioUdpSocket)> {
    let socket = UdpSocket::bind(addr)?;
    let socket_for_later = socket.try_clone()?;
    info!("Listening on: {}", socket.local_addr().unwrap());
    let tokio_socket = TokioUdpSocket::from_std(socket, reactor_handle)?;
    Ok((socket_for_later, tokio_socket))
}

pub fn spawn_server(
    addr: &str,
    runtime: &mut Runtime,
    pool: Arc<ThreadPool>,
    config: Arc<Config>,
) -> IoResult<()> {
    let addr = addr.parse().unwrap();
    let (udp_socket, tokio_socket) = get_sockets(addr, runtime.reactor())?;
    let server = Server {
        config: config,
        threadpool: pool,
        tokio_socket: tokio_socket,
        socket: udp_socket,
        buf: vec![0; 1500],
    };

    let server = server.map_err(|e| error!("server error = {:?}", e));
    runtime.spawn(server);

    Ok(())
}

pub fn handle_request(server: &Server, amt: usize, src_addr: SocketAddr) {
    match server.socket.try_clone() {
        Ok(socket) => {
            let wr = WorkerResources {
                config: server.config.clone(),
                src: RequestSource {
                    socket: socket,
                    addr: src_addr,
                },
                //only clone the bytes we really need
                buf: server.buf[..amt].to_vec().clone(),
                amt: amt,
            };
            spawn_work(server, wr);
        }
        Err(err) => {
            error!("could not clone UDP socket {}", err);
        }
    }
}

fn spawn_work(server: &Server, wr: WorkerResources) {
    server.threadpool.install(move || {
        parse_packet(wr);
    });
}
