use actix_web::{error, get, web, App, HttpResponse, HttpServer};
use anyhow::{Context, Result};
use async_std::io;
use http::StatusCode;
use serde::de;
use serde::Deserialize;
use serde::Deserializer;
use std::net::IpAddr;
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::net::{Shutdown, SocketAddr};
use std::time::Duration;
use structopt::clap::crate_version;
use structopt::StructOpt;

use crate::args::ProbyConfig;

mod args;

#[derive(Debug, Clone)]
struct SocketInfo {
    original_str: String,
    socket_addr: SocketAddr,
}

impl<'de> Deserialize<'de> for SocketInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let param = String::deserialize(deserializer)?;
        let mut socket_addrs = param
            .as_str()
            .to_socket_addrs()
            .map_err(|_| de::Error::custom("Error while parsing host or port"))?;
        Ok(SocketInfo {
            socket_addr: socket_addrs
                .next()
                .ok_or_else(|| de::Error::custom("Weird bug happened"))?,
            original_str: param,
        })
    }
}

#[derive(Clone)]
struct FormattedSockets {
    data: Vec<String>,
}

#[get("/")]
async fn usage(sockets: web::Data<FormattedSockets>) -> String {
    let examples: String = sockets
        .data
        .iter()
        .map(|x| format!("  curl http://{}/example.com:1337\n", x))
        .collect();
    format!(
        "proby {version}

Try something like this:

{examples}",
        version = crate_version!(),
        examples = examples,
    )
}

#[derive(Debug, Deserialize)]
struct HttpCode(#[serde(with = "serde_with::rust::display_fromstr")] StatusCode);

#[derive(Debug, Deserialize)]
struct CheckHostPortOptions {
    good: Option<HttpCode>,
    bad: Option<HttpCode>,
    timeout: Option<u64>,
}

#[get("/{socket_info}")]
async fn check_host_port(
    socket_info: web::Path<SocketInfo>,
    params: web::Query<CheckHostPortOptions>,
) -> HttpResponse {
    let good_status = params.good.as_ref().unwrap_or(&HttpCode(StatusCode::OK));
    let bad_status = params
        .bad
        .as_ref()
        .unwrap_or(&HttpCode(StatusCode::SERVICE_UNAVAILABLE));
    let timeout = Duration::new(params.timeout.unwrap_or(1), 0);

    let socket_addr = socket_info.socket_addr;
    if let Ok(stream) = web::block(move || TcpStream::connect_timeout(&socket_addr, timeout)).await {
        stream
            .shutdown(Shutdown::Both)
            .expect("Couldn't tear down TCP connection");
        let good_body = format!("{} is connectable", socket_info.original_str);
        HttpResponse::with_body(good_status.0, good_body.into())
    } else {
        let bad_body = format!("{} is NOT connectable", socket_info.original_str);
        HttpResponse::with_body(bad_status.0, bad_body.into())
    }
}

/// Convert a `Vec` of interfaces and a port to a `Vec` of `SocketAddr`.
fn interfaces_to_sockets(interfaces: &[IpAddr], port: u16) -> Result<Vec<SocketAddr>> {
    interfaces
        .iter()
        .map(|&interface| {
            if interface.is_ipv6() {
                // If the interface is IPv6 then we'll print it with brackets so that it is
                // clickable and also because for some reason, actix-web won't it otherwise.
                format!("[{}]", interface)
            } else {
                format!("{}", interface)
            }
        })
        .map(|interface| {
            format!("{interface}:{port}", interface = &interface, port = port,)
                .parse::<SocketAddr>()
        })
        .collect::<Result<Vec<SocketAddr>, std::net::AddrParseError>>()
        .context("Error during creation of sockets from interfaces and port")
}

#[actix_rt::main]
async fn main() -> Result<()> {
    let args = ProbyConfig::from_args();

    let socket_addresses = interfaces_to_sockets(&args.interfaces, args.port)?;

    let formatted_sockets = FormattedSockets {
        data: socket_addresses.iter().map(|x| x.to_string()).collect(),
    };

    println!(
        "proby {version}\n\nServing on:\n{sockets}",
        version = crate_version!(),
        sockets = formatted_sockets
            .data
            .iter()
            .map(|x| format!("http://{}\n", x))
            .collect::<String>()
    );
    HttpServer::new(move || {
        App::new()
            .data(formatted_sockets.clone())
            .service(usage)
            .service(check_host_port)
            .app_data(web::PathConfig::default().error_handler(|err, _req| {
                let err_text = err.to_string();
                error::InternalError::from_response(err, HttpResponse::BadRequest().body(err_text))
                    .into()
            }))
    })
    .bind(socket_addresses.as_slice())?
    .run()
    .await
    .context("Error while running web server!")
}
