use resolver::*;

pub struct Config {
    pub resolver: Resolver
}

impl Config {
    pub fn init_cloudflare() -> Self {
        Config{
            resolver: Resolver::new_cloudflare()
        }

    }

    pub fn init_google() -> Self {
        info!("Google dns requires that this dns proxy isn't its own dns server!");
        info!("Google dns is https://dns.google.com/, so the system always needs to be able to resolve that");
        //side note: thats because their cert doesn't stretch over their IPs (only to *.google.com)
        //Lets hope they fix that one day. TODO: this might not be a problem with with native tls
        //anymore. Test with Ip.
        Config{
            resolver: Resolver::new_google()
        }
    }

    // https://www.openresolve.com/
    // pub fn ini_openresolve() {
    //      //api.openresolve.com
    //      //same problem as google, their cert covers just the domain.
    //      //side node:
    //      //We could implement to use our own cert, but that would make the code really
    //      //no straight forward, for very little gained.
    //      //Also hyper-tls isn't really build for that, as far as i can tell.
    // }
}
