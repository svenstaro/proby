#![feature(lookup_host)]
#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

use std::time::Duration;
use std::net::{Shutdown, TcpStream, ToSocketAddrs, SocketAddr};

use rocket::State;
use rocket::http::{RawStr, Status};
use rocket::response::status;
use rocket::request::FromParam;

struct RocketConfig {
    hostname: String,
    port: u16,
}

struct SocketInfo {
    original_host: String,
    socket_addr: SocketAddr,
}

impl<'r> FromParam<'r> for SocketInfo {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        let mut socket_addrs = param.as_str().to_socket_addrs().map_err(|_| "Error while parsing host or port")?;
        Ok(SocketInfo {
            socket_addr: socket_addrs.next().ok_or("Weird bug happened")?,
            original_host: param.to_string(),
        })
    }
}


#[get("/")]
fn usage(rocket_config: State<RocketConfig>) -> String {
    format!("
proby v0.1.0
Try something like this:

    curl {host}:{port}/example.com/1337
    ", host=rocket_config.hostname, port=rocket_config.port)
}

#[get("/<socket_info>")]
fn check_host_port(socket_info: Result<SocketInfo, &RawStr>) -> status::Custom<String> {
    let socket_info = match socket_info {
        Ok(s) => s,
        Err(e) => return status::Custom(Status::UnprocessableEntity, e.to_string()),
    };
    if let Ok(stream) = TcpStream::connect_timeout(&socket_info.socket_addr, Duration::new(1, 0)) {
        stream.shutdown(Shutdown::Both).expect("Couldn't tear down TCP connection");
        status::Custom(Status::from_code(200).unwrap(), format!("{} is connectable", socket_info.original_host))
    } else {
        status::Custom(Status::from_code(400).unwrap(), format!("{} is NOT connectable", socket_info.original_host))
    }
}

fn main() {
    let rocket = rocket::ignite();

    let rocket_config = RocketConfig {
        hostname: rocket.config().clone().address,
        port: rocket.config().port,
    };
    rocket.manage(rocket_config).mount("/", routes![usage, check_host_port]).launch();
}
