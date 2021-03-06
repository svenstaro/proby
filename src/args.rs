use std::net::IpAddr;
use structopt::StructOpt;

/// Checks wether an interface is valid, i.e. it can be parsed into an IP address
fn parse_interface(src: &str) -> Result<IpAddr, std::net::AddrParseError> {
    src.parse::<IpAddr>()
}

#[derive(StructOpt, Clone, Debug)]
#[structopt(
    name = "proby",
    author,
    about,
    global_settings = &[structopt::clap::AppSettings::ColoredHelp],
)]
pub struct ProbyConfig {
    /// Be quiet (log nothing)
    #[structopt(short, long)]
    pub quiet: bool,

    /// Be verbose (log data of incoming and outgoing requests). If given twice it will also log
    /// the body data.
    #[structopt(short, long, conflicts_with = "quiet")]
    pub verbose: bool,

    /// Interface to bind to
    #[structopt(
        short,
        long,
        parse(try_from_str = parse_interface),
        number_of_values = 1,
        default_value = "0.0.0.0"
    )]
    pub interfaces: Vec<IpAddr>,

    /// Port on which to listen
    #[structopt(short, long, default_value = "8080")]
    pub port: u16,
}
