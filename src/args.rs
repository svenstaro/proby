use clap::Parser;
use std::net::IpAddr;

/// Checks wether an interface is valid, i.e. it can be parsed into an IP address
fn parse_interface(src: &str) -> Result<IpAddr, std::net::AddrParseError> {
    src.parse::<IpAddr>()
}

#[derive(Parser, Clone, Debug)]
#[command(name = "proby", author, about, version)]
pub struct ProbyConfig {
    /// Be quiet (log nothing)
    #[arg(short, long)]
    pub quiet: bool,

    /// Be verbose (log data of incoming and outgoing requests). If given twice it will also log
    /// the body data.
    #[arg(short, long, conflicts_with = "quiet")]
    pub verbose: bool,

    /// Interface to bind to
    #[arg(
        short,
        long,
        value_parser = parse_interface,
        number_of_values = 1,
        default_value = "0.0.0.0"
    )]
    pub interfaces: Vec<IpAddr>,

    /// Port on which to listen
    #[arg(short, long, default_value = "8080")]
    pub port: u16,
}
