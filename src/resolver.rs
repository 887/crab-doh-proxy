use std::io::{self, Write};
use std::net::{SocketAddr, UdpSocket};
use std::env;

use hyper::Client;
use hyper::rt::{self, Future, Stream, lazy};
use hyper::Uri;

trait DohAddress {
    /// Build URI to resolve dns request for
    fn get_address(&self) -> &'static str;
    /// Checks if DNS resolver address is an IP or needs DNS resolution itself
    fn is_ip_address_string(&self) -> bool;
}

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

    fn get_doh_addres(&self) -> &'static str {
        match self {
            Resolver::Cloudflare() => {"https://1.1.1.1/dns-query?"},
            Resolver::Google() => {"https://dns.google.com/resolve?"}
        }
    }

    /// Checks if DNS resolver address is an IP or needs DNS resolution itself
    pub fn is_ip_address_string(&self) -> bool {
        match self {
            Resolver::Cloudflare() => {true},
            Resolver::Google() => {false}
        }
    }

    /// Build URI to resolve dns request for
    pub fn get_url(&self, _type: u16, name: &str) -> Uri {
        //TODO: take the name and type, put it in the url
        // https://developers.google.com/speed/public-dns/docs/dns-over-https
        // let request = format!("GET /resolve?name={}&type={}&dnssec=true HTTP/1.0\r\nHost: \

        let addr = self.get_doh_addres();
        addr.parse::<Uri>().unwrap()
    }
}


