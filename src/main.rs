#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;

use std::time::Duration;
use std::net::{Shutdown, TcpStream, ToSocketAddrs, SocketAddr};

use rocket::State;
use rocket::http::{RawStr, Status};
use rocket::response::status;
use rocket::request::{FromParam, FromFormValue};

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
            },
            _ => Err(form_value),
        }
    }
}

#[derive(FromForm, Clone, Debug)]
struct QueryOptions {
    good: Option<HTTPStatus>,
    bad: Option<HTTPStatus>,
    timeout: Option<u64>,
}

#[get("/")]
fn usage(rocket_config: State<RocketConfig>) -> String {
    format!("
proby v0.1.1
Try something like this:

    curl {host}:{port}/example.com:1337
    ", host=rocket_config.hostname, port=rocket_config.port)
}

// This route duplication should get better in rocket 0.4 (https://github.com/SergioBenitez/Rocket/issues/608)
#[get("/<socket_info>")]
fn check_host_port_default(socket_info: Result<SocketInfo, &RawStr>) -> status::Custom<String> {
    let query_opts = QueryOptions {
        good: Some(HTTPStatus(Status::Ok)),
        bad: Some(HTTPStatus(Status::BadRequest)),
        timeout: Some(1),
    };
    check_host_port(socket_info, Some(query_opts))
}

#[get("/<socket_info>?<query_opts>")]
fn check_host_port(socket_info: Result<SocketInfo, &RawStr>, query_opts: Option<QueryOptions>) -> status::Custom<String> {
    let socket_info = match socket_info {
        Ok(s) => s,
        Err(e) => return status::Custom(Status::UnprocessableEntity, e.to_string()),
    };

    let (HTTPStatus(good_status), HTTPStatus(bad_status), timeout) = if let Some(qopts) = query_opts {
        let good = match qopts.good {
            Some(good) => good,
            None => HTTPStatus(Status::Ok),
        };
        let bad = match qopts.bad {
            Some(bad) => bad,
            None => HTTPStatus(Status::BadRequest),
        };
        let timeout = match qopts.timeout {
            Some(timeout) => timeout,
            None => 1,
        };
        (good, bad, timeout)
    }
    else {
        (HTTPStatus(Status::Ok), HTTPStatus(Status::BadRequest), 1)
    };

    if let Ok(stream) = TcpStream::connect_timeout(&socket_info.socket_addr, Duration::new(timeout, 0)) {
        stream.shutdown(Shutdown::Both).expect("Couldn't tear down TCP connection");
        status::Custom(good_status, format!("{} is connectable", socket_info.original_host))
    } else {
        status::Custom(bad_status, format!("{} is NOT connectable", socket_info.original_host))
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
        .mount("/", routes![usage, check_host_port_default, check_host_port])
        .launch();
}
