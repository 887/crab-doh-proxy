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
