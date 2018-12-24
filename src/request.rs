use std::sync::Arc;

use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::env;

use rayon::ThreadPool;

use dns_parser::{Builder, Class, Header, Opcode, Packet, QueryClass, QueryType, ResponseCode, Type};

use native_tls::{TlsConnector, TlsStream};

use config::Config;

use dns::*;

use serde_json::from_str as serde_json_from_str;

use dns_response_builder::{remove_fqdn_dot, ResponseBuilder};

pub struct RequestSource {
    pub socket: UdpSocket,
    pub addr: SocketAddr,
}

pub struct WorkerResources {
    pub config: Arc<Config>,
    pub src: RequestSource,
    pub buf: Vec<u8>,
    pub amt: usize,
}

struct RequestSourceAndPacket<'a> {
    src: RequestSource,
    packet: Packet<'a>,
}

pub fn parse_packet(wr: WorkerResources) {
    //only print here in the thread, so we dont block stdio on the udp receiving thread
    debug!("Received {} bytes from {}", wr.amt, wr.src.addr);
    // https://tailhook.github.io/dns-parser/dns_parser/struct.Packet.html
    if let Ok(packet) = Packet::parse(&wr.buf) {
        // only support one question
        // https://groups.google.com/forum/#!topic/comp.protocols.dns.bind/uOWxNkm7AVg
        if packet.questions.len() != 1 {
            error!(
                "Invalid request from {}, packet questions != 1 (amt: {})",
                wr.src.addr,
                packet.questions.len()
            );
        } else {
            debug!("packet parsed!");
            make_request(
                &wr.config,
                &RequestSourceAndPacket {
                    src: wr.src,
                    packet,
                },
            );
        }
    } else {
        debug!("Invalid request from {}", wr.src.addr);
    }
}

fn make_request(config: &Arc<Config>, rs: &RequestSourceAndPacket) {
    let qtype = rs.packet.questions[0].qtype as u16;
    let qname = rs.packet.questions[0].qname.to_string();
    let id = rs.packet.header.id;
    debug!("requested id:{} name:{}, type:{}", id, qname, qtype);

    let addr = config.resolver.get_addr();

    trace!("connecting to {}", addr);
    match TcpStream::connect(addr) {
        Ok(tcp_stream) => {
            trace!("connection established");
            connect_tls(&config, qtype, &qname, &rs, tcp_stream);
        }
        Err(err) => {
            error!("tcp connection failed {:?}", err);
        }
    }
}

fn connect_tls(
    config: &Arc<Config>,
    qtype: u16,
    qname: &str,
    rs: &RequestSourceAndPacket,
    tcp_stream: TcpStream,
) {
    let domain = config.resolver.get_domain();
    let connector = TlsConnector::builder().build().unwrap();
    trace!("upgrading to tls");
    match connector.connect(domain, tcp_stream) {
        Ok(tls_stream) => {
            trace!("upgrade success");
            run_request(config, qtype, qname, &rs, tls_stream);
        }
        Err(err) => {
            error!("tls connection failed {:?}", err);
        }
    }
}

fn run_request(
    config: &Arc<Config>,
    qtype: u16,
    qname: &str,
    rs: &RequestSourceAndPacket,
    mut tls_stream: TlsStream<TcpStream>,
) {
    use httparse::{Response, Status, EMPTY_HEADER};

    let request = config.resolver.get_request(qtype, &qname);

    trace!("writing data");
    tls_stream.write_all(request.as_str().as_bytes()).unwrap();
    let mut res = vec![];
    tls_stream.read_to_end(&mut res).unwrap();

    trace!("data read");
    let mut headers = [EMPTY_HEADER; 16];
    let mut response = Response::new(&mut headers);

    match response.parse(&res) {
        Ok(response_status) => match response.code {
            Some(code) => {
                if code == 200 {
                    match response_status {
                        Status::Complete(header_length) => {
                            let body = res[header_length..].to_vec();
                            parse_response_body(config, &rs, &body);
                        }
                        incomplete => {
                            error!("incomplete response {:?}", incomplete);
                        }
                    }
                } else {
                    error!("response code = {:?}", code);
                }
            }
            None => {
                error!("no response code");
            }
        },
        Err(err) => {
            error!("parsing response failed {:?}", err);
        }
    }
}

fn parse_response_body(config: &Arc<Config>, rs: &RequestSourceAndPacket, res_body: &[u8]) {
    let res_body_string = String::from_utf8_lossy(&res_body);
    trace!("response string: {}", res_body_string);

    if let Ok(deserialized) = serde_json_from_str::<DnsRequest>(&res_body_string) {
        trace!("response json deserialized: {:?}", deserialized);
        build_response(config, &rs, deserialized)
    } else {
        error!("couldn't deserialize json");
    }
}

fn build_response(config: &Arc<Config>, rs: &RequestSourceAndPacket, deserialized: DnsRequest) {
    let id = rs.packet.header.id;
    let qtype = rs.packet.questions[0].qtype as u16;
    let qname = rs.packet.questions[0].qname.to_string();

    let mut response = ResponseBuilder::new_response(
        id,
        ResponseCode::NoError,
        deserialized.tc,
        deserialized.rd,
        deserialized.ra,
    );

    for question in deserialized.questions {
        let query_type = QueryType::parse(question.qtype).unwrap();
        response.add_question(
            &remove_fqdn_dot(&question.qname),
            query_type,
            QueryClass::IN,
        );
    }

    info!(
        "requested id:{} name:{}, type:{}, response: {:?}",
        rs.packet.header.id, qname, qtype, deserialized.answers
    );

    if let Some(answers) = deserialized.answers {
        for answer in answers {
            if let Ok(data) = answer.write() {
                response.add_answer(
                    &remove_fqdn_dot(&answer.aname),
                    Type::parse(answer.atype).unwrap(),
                    Class::IN,
                    answer.ttl,
                    data,
                );
            }
        }
    }

    let data = match response.build() {
        Ok(data) | Err(data) => data,
    };

    send_response(&rs.src, &data);
}

fn send_response(rs: &RequestSource, buf: &[u8]) {
    let amt = buf.len();
    match rs.socket.send_to(&buf, &rs.addr) {
        Ok(_) => {
            debug!("Responded with {:?} bytes to {}", amt, rs.addr);
        }
        Err(_) => {
            error!("Failed to send {:?} bytes to {}", amt, rs.addr);
        }
    };
}
