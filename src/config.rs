use resolver::*;

pub struct Config {
    pub resolver: Resolver,
}

impl Config {
    pub fn init_cloudflare() -> Self {
        Config {
            resolver: Resolver::new_cloudflare(),
        }
    }

    pub fn init_google() -> Self {
        Config {
            resolver: Resolver::new_google(),
        }
    }

    // TODO:
    // https://www.openresolve.com/
    // pub fn ini_openresolve() {
    //      //api.openresolve.com
    // }
}
