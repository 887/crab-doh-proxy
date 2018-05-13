use std::io::{self, Write};
use std::net::{SocketAddr, UdpSocket};
use std::env;

use hyper::Client;
use hyper::rt::{self, Future, Stream, lazy};
use hyper::Uri;

pub enum Resolver {
    //cloudlfare, also needs the content-type header for application/dns-json
    //"1.1.1.1" ,1.0.0.1, dns.cloudflare.com
    // -> /dns-query?{name}{type}
    Cloudflare(CloudflareResolver),
    //or the good old google
    //216.58.195.78, dns.google.com
    // -> /resolve?{name}{type}
    Google(GoogleResolver),
}

impl Resolver {
    pub fn new_cloudflare() -> Self {
        Resolver::Cloudflare(
            CloudflareResolver{
                addr: "https://1.1.1.1/dns-query?"
            })
    }
    pub fn new_google() -> Self {

        Resolver::Google(
            GoogleResolver{
                addr: "https://dns.google.com/resolve?"
            })
    }

    pub fn get_url(&self, _type: u16, name: &str) -> Uri {
        //TODO: take the name and type, put it in the url
        // https://developers.google.com/speed/public-dns/docs/dns-over-https
        // let request = format!("GET /resolve?name={}&type={}&dnssec=true HTTP/1.0\r\nHost: \

        match self {
            Resolver::Cloudflare(resolver) => {
                resolver.addr.parse::<Uri>().unwrap()
            },
            Resolver::Google(resolver) => {
                resolver.addr.parse::<Uri>().unwrap()
            }
        }
    }
}

pub struct CloudflareResolver {
    addr: &'static str,
}

pub struct GoogleResolver {
    addr: &'static str,
}

