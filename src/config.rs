use resolver::*;

pub struct Config {
    pub resolver: Resolver
}

impl Config {
    pub fn init_cloudflare() -> Self {
        //cloudflares cert for their dns server covers 1.1.1.1 we are all good here
        Config{
            resolver: Resolver::new_cloudflare()
        }

    }

    pub fn init_google() -> Self {
        info!("Google dns requires that this dns proxy isn't its own dns server!")
        info!("Google dns is https://dns.google.com/, so the system always needs to be able to resolve that,
              to use this proxy here.")
        //side note: thats because their cert doesn't stretch over their Ips but only to *.google.com
        //Lets hope they fix that.
        Config{
            resolver: Resolver::new_google()
        }
    }

    // https://www.openresolve.com/
    // pub fn ini_openresolve() {
    //      //api.openresolve.com
    //      //same problem as google, their cert covers just the domain.
    //      //side node:
    //      //We could implement using our own cert but that would make the code really
    //      //no straight forward, for very little gained and hyper-tls isn't really build for that,
    //      //as far as i can tell.
    // }
}
