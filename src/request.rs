use std::sync::Arc;

use std::io::{self, Read, Write};
use std::net::{SocketAddr, UdpSocket, TcpStream};
use std::env;

use rayon::{ThreadPool};

use dns_parser::{Builder, Class, Packet, QueryClass, QueryType, ResponseCode, Type};

use futures::{future};

use native_tls::TlsConnector;

use server::Server;
use config::Config;

struct Source {
    socket: UdpSocket,
    src_addr: SocketAddr,
}

pub fn handle_request(server: &mut Server, amt: usize, src_addr: SocketAddr) -> Result<(), io::Error> {
    //TODO: make tcp request to doh-server, parse reult, reply in new thread on the server socket.

    //only clone the bytes we really need
    let buf = server.buf[..amt].to_vec().clone(); //clones this buffer
    //clone it again with try_clone for lifetime requirements!
    let socket = server.socket.try_clone()?;
    let config = server.config.clone();
    server.threadpool.install(move || {
        let buf = if amt < 14 {
            mock_request()
        } else {
            buf
        };

        parse_packet(config, socket, src_addr, buf, amt);
    });

    Ok(())
}

fn mock_request() -> Vec<u8> {
    debug!("mocking request because length < 14");
    let mut b = Builder::new_query(0, false);
    b.add_question("google.com", QueryType::A, QueryClass::Any);
    match b.build() {
        Ok(data) | Err(data) => data,
    }
}

fn parse_packet(config: Arc<Config>, socket: UdpSocket, src_addr: SocketAddr, buf: Vec<u8>, amt: usize) {
    //only print here in the thread, so we dont block stdio on the udp receiving thread
    debug!("Received {} bytes from {}", amt, src_addr);
    let rs = Source {socket: socket, src_addr: src_addr};

    // https://tailhook.github.io/dns-parser/dns_parser/struct.Packet.html
    if let Ok(packet) = Packet::parse(&buf) {
        // only support one question
        // https://groups.google.com/forum/#!topic/comp.protocols.dns.bind/uOWxNkm7AVg
        if packet.questions.len() != 1 {
            error!("Invalid request from {}, packet questions != 1 (amt: {})", rs.src_addr, packet.questions.len());
        } else {
            debug!("packet parsed!");
            make_request(config, rs, packet);
            // handle_packet(config, cert, receiver, packet);
        }
    } else {
        debug!("Invalid request from {}", rs.src_addr);
    }
}

fn make_request(config: Arc<Config>, rs: Source, packet: Packet) {
    //build_response
    // let addr = "127.0.0.1:6142".parse().unwrap();
    // let stream = addr.connect_async::<tokio_tls::TlsStream>();

    let qtype = packet.questions[0].qtype as u16;
    let qname = packet.questions[0].qname.to_string();
    info!("requested name:{}, type:{}", qname, qtype);


    let connector = TlsConnector::builder().unwrap().build().unwrap();

    let addr = config.resolver.get_addr();
    let domain = config.resolver.get_domain();

    let stream = TcpStream::connect(addr).unwrap();
    let mut stream = connector.connect(domain, stream).unwrap();

    let url = config.resolver.get_url(qtype, &qname);

    stream.write_all(b"GET / HTTP/1.0\r\n\r\n").unwrap();
    let mut res = vec![];
    stream.read_to_end(&mut res).unwrap();
    println!("{}", String::from_utf8_lossy(&res));

    let buf = vec![0;1500];
    send_response(rs, buf);
}

// fn build_response(rs: Source, packet: Packet, deserialized: Request) {
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

fn send_response(rs: Source, buf: Vec<u8>) {
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

