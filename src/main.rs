use std::fmt::Write;
use std::net::{IpAddr, Shutdown, SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

use actix_web::{error, get, web, App, HttpRequest, HttpResponse, HttpServer};
use anyhow::{Context, Result};
use http::StatusCode;
use log::info;
use serde::de;
use serde::Deserialize;
use serde::Deserializer;
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
        Ok(Self {
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
    let examples: String = sockets.data.iter().fold(String::new(), |mut output, x| {
        let _ = writeln!(output, "  curl http://{x}/example.com:1337");
        output
    });
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
    args: web::Data<ProbyConfig>,
    req: HttpRequest,
    socket_info: web::Path<SocketInfo>,
    params: web::Query<CheckHostPortOptions>,
) -> HttpResponse {
    let good_status = params.good.as_ref().unwrap_or(&HttpCode(StatusCode::OK));
    let bad_status = params
        .bad
        .as_ref()
        .unwrap_or(&HttpCode(StatusCode::SERVICE_UNAVAILABLE));
    let timeout = Duration::new(params.timeout.unwrap_or(1), 0);

    if args.verbose {
        let params_text = format!(
            "(good: {}, bad: {}, timeout: {})",
            good_status.0.as_u16(),
            bad_status.0.as_u16(),
            timeout.as_secs()
        );
        info!(
            "{} requesting check of {} {}",
            req.peer_addr().unwrap(),
            socket_info.original_str,
            params_text,
        );
    }

    let socket_addr = socket_info.socket_addr;
    if let Ok(stream) = web::block(move || TcpStream::connect_timeout(&socket_addr, timeout)).await
    {
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

#[actix_web::main]
async fn main() -> Result<()> {
    let args = ProbyConfig::from_args();

    let socket_addresses = interfaces_to_sockets(&args.interfaces, args.port)?;

    let formatted_sockets = FormattedSockets {
        data: socket_addresses.iter().map(|x| x.to_string()).collect(),
    };

    let log_level = if args.quiet {
        simplelog::LevelFilter::Error
    } else {
        simplelog::LevelFilter::Info
    };

    if simplelog::TermLogger::init(
        log_level,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .is_err()
    {
        simplelog::SimpleLogger::init(log_level, simplelog::Config::default())
            .expect("Couldn't initialize logger")
    }

    info!("proby {version}", version = crate_version!(),);
    HttpServer::new(move || {
        App::new()
            .data(args.clone())
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
