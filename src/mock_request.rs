use std::sync::Arc;

use std::io::{self, Read, Result as IoResult, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::env;

use dns_parser::{Builder, Class, Packet, QueryClass, QueryType, ResponseCode, Type};

use config::Config;

use request::{parse_packet, RequestSource, WorkerResources};

fn mock_query() -> Vec<u8> {
    let mut b = Builder::new_query(0, false);
    b.add_question("google.com", QueryType::A, QueryClass::Any);
    match b.build() {
        Ok(data) | Err(data) => data,
    }
}

pub fn mock_request() -> IoResult<()> {
    let addr = "127.0.0.1:6142".parse().unwrap();
    let socket = UdpSocket::bind(addr)?;
    let buf = mock_query();
    let amt = buf.len();
    let wr = WorkerResources {
        config: Arc::new(Config::init_google()),
        src: RequestSource {
            socket: socket,
            addr: addr,
        },
        buf: buf,
        amt: amt,
    };
    parse_packet(wr);
    Ok(())
}
