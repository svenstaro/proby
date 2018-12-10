#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::net::{Shutdown, SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

use rocket::http::{RawStr, Status};
use rocket::request::{FromFormValue, FromParam};
use rocket::response::status;
use rocket::State;

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
        let mut socket_addrs = param
            .as_str()
            .to_socket_addrs()
            .map_err(|_| "Error while parsing host or port")?;
        Ok(SocketInfo {
            socket_addr: socket_addrs.next().ok_or("Weird bug happened")?,
            original_host: param.to_string(),
        })
    }
}

#[derive(Clone, Debug)]
struct HTTPStatus(Status);

impl<'v> FromFormValue<'v> for HTTPStatus {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<HTTPStatus, &'v RawStr> {
        match form_value.parse::<u16>() {
            Ok(code) => {
                if let Some(status) = Status::from_code(code) {
                    Ok(HTTPStatus(status))
                } else {
                    Err(RawStr::from_str("Invalid HTTP status code"))
                }
            }
            _ => Err(form_value),
        }
    }
}

#[get("/")]
fn usage(rocket_config: State<RocketConfig>) -> String {
    format!(
        "proby v0.1.4

Try something like this:

    curl {host}:{port}/example.com:1337",
        host = rocket_config.hostname,
        port = rocket_config.port
    )
}

#[get("/<socket_info>?<good>&<bad>&<timeout>")]
fn check_host_port(
    socket_info: Result<SocketInfo, &RawStr>,
    good: Option<HTTPStatus>,
    bad: Option<HTTPStatus>,
    timeout: Option<u64>,
) -> status::Custom<String> {
    let socket_info = match socket_info {
        Ok(s) => s,
        Err(e) => return status::Custom(Status::UnprocessableEntity, e.to_string()),
    };

    let HTTPStatus(good_status) = good.unwrap_or(HTTPStatus(Status::Ok));
    let HTTPStatus(bad_status) = bad.unwrap_or(HTTPStatus(Status::ServiceUnavailable));
    let timeout = timeout.unwrap_or(1);

    if let Ok(stream) =
        TcpStream::connect_timeout(&socket_info.socket_addr, Duration::new(timeout, 0))
    {
        stream
            .shutdown(Shutdown::Both)
            .expect("Couldn't tear down TCP connection");
        status::Custom(
            good_status,
            format!("{} is connectable", socket_info.original_host),
        )
    } else {
        status::Custom(
            bad_status,
            format!("{} is NOT connectable", socket_info.original_host),
        )
    }
}

fn main() {
    let rocket = rocket::ignite();

    let rocket_config = RocketConfig {
        hostname: rocket.config().clone().address,
        port: rocket.config().port,
    };
    rocket
        .manage(rocket_config)
        .mount("/", routes![usage, check_host_port])
        .launch();
}
