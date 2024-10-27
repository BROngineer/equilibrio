use std::net::SocketAddr;
use clap::{Arg, Command};

pub struct Config {
    pub bind_address: SocketAddr,
    pub endpoint: SocketAddr,
}

pub fn parse() -> Config {
    let matches = Command::new("equilibrio")
        .version("0.1.0")
        .arg(
            Arg::new("bind-address")
                .long("bind-address")
                .required(false)
                .default_value("127.0.0.1")
        )
        .arg(Arg::new("bind-port")
            .long("bind-port")
            .required(false)
            .default_value("9080")
        )
        .arg(Arg::new("endpoint")
            .short('e')
            .long("endpoint")
            .required(true)
        ).get_matches();

    let bind_addr = matches.get_one::<String>("bind-address").expect("Missing bind-address argument").to_string();
    let bind_port = matches.get_one::<String>("bind-port").expect("Missing bind-port argument").to_string();
    let endpoint_addr = matches.get_one::<String>("endpoint").expect("Missing endpoint argument").to_string();

    let bind_address = format!("{}:{}", bind_addr, bind_port).parse().expect("Invalid bind address");
    let endpoint = endpoint_addr.parse().expect("Invalid endpoint address");
    Config {
        bind_address,
        endpoint,
    }
}