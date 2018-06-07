#![feature(lookup_host)]
#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

use std::time::Duration;
use std::net::{Shutdown, TcpStream, ToSocketAddrs, SocketAddr};
use std::error::Error;

use rocket::State;
use rocket::http::{RawStr, Status};
use rocket::response::status;
use rocket::request::FromParam;

struct RocketConfig {
    hostname: String,
    port: u16,
}

struct SocketAddrToSocketAddrs(SocketAddr);

impl<'r> FromParam<'r> for SocketAddrToSocketAddrs {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        let mut socket_addrs = param.as_str().to_socket_addrs().map_err(|_| param)?;
        Ok(SocketAddrToSocketAddrs(socket_addrs.next().ok_or("Weird bug happened")?))
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

#[get("/<host_socket>")]
fn check_host_port(host_socket: SocketAddrToSocketAddrs) -> Result<status::Custom<String>, String> {
    // let mut ip_addr = (hostname.as_str(), port).to_socket_addrs().unwrap();
    // let socket_addr = ip_addr.next().unwrap();
    let SocketAddrToSocketAddrs(socket_addr) = host_socket;
    if let Ok(stream) = TcpStream::connect_timeout(&socket_addr, Duration::new(1, 0)) {
        stream.shutdown(Shutdown::Both).expect("Couldn't tear down TCP connection");
        Ok(status::Custom(Status::from_code(200).unwrap(), "Host is connectable on port".to_owned()))
    } else {
        Ok(status::Custom(Status::from_code(400).unwrap(), "Host is not connectable on port".to_owned()))
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
