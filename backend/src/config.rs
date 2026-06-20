use std::net::{IpAddr, Ipv4Addr};

pub const DEFAULT_MAX_BODY_SIZE: usize = 50 * 1000 * 1000;

pub struct Config {
    pub host: IpAddr,
    pub port: u16,
    pub max_body_size_byte: usize,
}

impl Config {
    pub fn load() -> Config {
        let host = std::env::var("QUIZLER_HOST")
            .map(|value| {
                value
                    .parse::<IpAddr>()
                    .expect("Provided QUIZLER_HOST was not a address")
            })
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));

        let port: u16 = std::env::var("QUIZLER_PORT")
            .map(|value| {
                value
                    .parse::<u16>()
                    .expect("Provided QUIZLER_PORT was not a valid port")
            })
            .unwrap_or(80);

        let max_body_size_byte: usize = std::env::var("QUIZLER_MAX_BODY_SIZE_BYTES")
            .map(|value| {
                value
                    .parse::<usize>()
                    .expect("Provided QUIZLER_MAX_BODY_SIZE_BYTES was not a valid unsigned integer")
            })
            .unwrap_or(DEFAULT_MAX_BODY_SIZE); // Default max size of 50mb

        Config {
            host,
            port,
            max_body_size_byte,
        }
    }
}
