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
            // Resolver::Google() => {"dns.google.com:443"} // this requires the proxy not to be
            // its own dns server
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

    pub fn get_dir(&self) -> &'static str {
        match self {
            Resolver::Cloudflare() => {"/dns-query"},
            Resolver::Google() => {"/resolve"}
        }
    }

    fn get_additional_params(&self) -> &'static str {
        match self {
            //TODO: parse from packet
            //&edns_client_subnet = edns

            Resolver::Cloudflare() => {"&dnssec=true&ct=application/dns-json"},
            Resolver::Google() => {"&dnssec=true"}
        }
    }

    fn get_headers(&self) -> &'static str {
        let pad_chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~";
        match self {
            //TODO: insert a random header here to pad the request and
            // \r\nPadding: random_string_on_every_request
            Resolver::Cloudflare() => {""},
            Resolver::Google() => {""}
        }
    }

    pub fn get_url(&self, _type: u16, name: &str) -> &'static str {
        //TODO: take the name and type, put it in the url
        // https://developers.google.com/speed/public-dns/docs/dns-over-https
        // let request = format!("GET /resolve?name={}&type={}&dnssec=true HTTP/1.0\r\nHost: \

        self.get_dir()
    }

    pub fn get_request(&self, _type: u16, name: &str) -> String {
        format!("GET {}?name={}&type={}{} HTTP/1.0\r\nHost: \
                {}{}\r\n\r\n",
                self.get_dir(),
                name,
                _type,
                self.get_additional_params(),
                self.get_domain(),
                self.get_headers())
    }
}


