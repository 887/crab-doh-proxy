use std::io::{self, Write};
use std::net::{SocketAddr, UdpSocket};
use std::env;

pub enum Resolver {
    //cloudlfare, also needs the content-type header for application/dns-json
    //"1.1.1.1" ,1.0.0.1, dns.cloudflare.com
    // -> /dns-query?{name}{type}
    Cloudflare(),
    //or the good old google
    //216.58.195.78, dns.google.com
    // -> /resolve?{name}{type}
    Google(),
}

impl Resolver {
    pub fn new_cloudflare() -> Self {
        Resolver::Cloudflare()
    }
    pub fn new_google() -> Self {
        Resolver::Google()
    }

    pub fn get_addr(&self) -> &'static str {
        match self {
            Resolver::Cloudflare() => {"1.1.1.1:443"},
            Resolver::Google() => {"216.58.195.78:443"}
        }
    }

    /// From the native-tls docs:
    /// The provided domain will be used for both SNI and certificate hostname
    /// validation.
    pub fn get_domain(&self) -> &'static str {
        match self {
            Resolver::Cloudflare() => {"1.1.1.1"},
            Resolver::Google() => {"dns.google.com"}
        }
    }

    fn get_doh(&self) -> &'static str {
        match self {
            Resolver::Cloudflare() => {"https://1.1.1.1/dns-query?"},
            Resolver::Google() => {"https://dns.google.com/resolve?"}
        }
    }

    pub fn get_url(&self, _type: u16, name: &str) -> &'static str {
        //TODO: take the name and type, put it in the url
        // https://developers.google.com/speed/public-dns/docs/dns-over-https
        // let request = format!("GET /resolve?name={}&type={}&dnssec=true HTTP/1.0\r\nHost: \

        self.get_doh()
    }
}


