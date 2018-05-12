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
extern crate tokio_tls;

mod dns;

use std::io::prelude::*;
use std::env;
use std::net::{SocketAddr, UdpSocket};
use std::io::{self, Write};

use tokio::net::UdpSocket as TokioUdpSocket;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::reactor::Handle;
use tokio::runtime::Runtime;

use tokio_tls::{TlsConnectorExt, TlsAcceptorExt};

use rayon::{ThreadPool, ThreadPoolBuilder};


use dns_parser::{Builder, Class, Packet, QueryClass, QueryType, ResponseCode, Type};

use log::{SetLoggerError, LevelFilter};

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
                    handle_request(self, amt, scr_addr)?;
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

fn handle_request(server: &mut Server, amt: usize, src_addr: SocketAddr) -> Result<(), io::Error> {

    //TODO: make tcp request to doh-server, parse reult, reply in new thread on the server socket.

    //clone it again with try_clone for lifetime requirements!
    let socket = server.socket.try_clone()?;
    //only clone the bytes we really need
    let buf = server.buf[..amt].to_vec().clone(); //clones this buffer
    server.threadpool.install(move || {
        parse_packet(socket, src_addr, buf, amt);
    });

    Ok(())
}

fn parse_packet(socket: UdpSocket, src_addr: SocketAddr, buf: Vec<u8>, amt: usize) {
    //only print here in the thread, so we dont block stdio on the udp receiving thread
    debug!("Received {} bytes from {}", amt, src_addr);
    let rs = RequestSource {socket: socket, src_addr: src_addr};

    // https://tailhook.github.io/dns-parser/dns_parser/struct.Packet.html
    if let Ok(packet) = Packet::parse(&buf) {
        // only support one question
        // https://groups.google.com/forum/#!topic/comp.protocols.dns.bind/uOWxNkm7AVg
        if packet.questions.len() != 1 {
            error!("Invalid request from {}, packet questions != 1 (amt: {})", rs.src_addr, packet.questions.len());
        } else {
            debug!("packet parsed!");
            make_request(rs, packet);
            // handle_packet(config, cert, receiver, packet);
        }
    } else {
        debug!("Invalid request from {}", rs.src_addr);
    }
}

fn make_request(rs: RequestSource, packet: Packet) {
    //build_response

    let buf = vec![0;1500];
    send_response(rs, buf);
}

// fn build_response(rs: RequestSource, packet: Packet, deserialized: Request) {
//
//         // apparently this part was already done:
//         // https://github.com/gmosley/rust-DNSoverHTTPS
//         // https://david-cao.github.io/rustdocs/dns_parser/
//
//         // the only reason to keep the incoming packet around is this id, maybe drop the rest?
//         let mut response = Builder::new_response(packet.id,
//                                                  ResponseCode::NoError,
//                                                  deserialized.tc,
//                                                  deserialized.rd,
//                                                  deserialized.ra);
//
//         for question in deserialized.questions {
//             let query_type = QueryType::parse(question.qtype).unwrap();
//             response.add_question(&remove_fqdn_dot(&question.qname),
//             query_type,
//             QueryClass::IN);
//         }
//
//         if let Some(answers) = deserialized.answers {
//             for answer in answers {
//                 if let Ok(data) = answer.write() {
//                     response.add_answer(&remove_fqdn_dot(&answer.aname),
//                     Type::parse(answer.atype).unwrap(),
//                     Class::IN,
//                     answer.ttl,
//                     data);
//                 }
//             }
//         }
//
//         let data = match response.build() {
//             Ok(data) | Err(data) => data,
//         };
//
//         SocketSender::new((receiver, data)).boxed()
// }

fn send_response(rs: RequestSource, buf: Vec<u8>) {
    let amt = buf.len();
    match rs.socket.send_to(&buf, &rs.src_addr) {
        Ok(_) => {
            debug!("Echoed {:?}/{} bytes to {}", amt, amt, rs.src_addr);
        }
        Err(_) => {
            debug!("Failed to send {:?}/{} bytes to {}", amt, amt, rs.src_addr);
        }
    };
}

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

fn main() -> std::io::Result<()> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

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
