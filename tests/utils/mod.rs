use assert_cmd::prelude::*;
use port_check::{free_local_port, is_port_reachable};
use std::ffi::OsStr;
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
pub struct ProbyProcess {
    pub child: Child,
    pub host: String,
    pub port: String,
    pub url: String,
}

impl Drop for ProbyProcess {
    fn drop(&mut self) {
        if let Err(e) = self.child.kill() {
            eprintln!("WARN: {}", e);
        }
    }
}

#[allow(dead_code)]
impl ProbyProcess {
    /// Get a Dummyhttp instance on a free port.
    pub fn new<I, S>(args: I) -> Result<ProbyProcess, Error>
    where
        I: IntoIterator<Item = S> + Clone + std::fmt::Debug,
        S: AsRef<OsStr> + PartialEq + From<&'static str>,
    {
        let host = "127.0.0.1".to_string();
        let port = free_local_port()
            .expect("Couldn't find a free local port")
            .to_string();

        let child = Command::cargo_bin("proby")?
            .arg("-p")
            .arg(&port)
            .args(args.clone())
            .stdout(Stdio::piped())
            .spawn()?;

        // Wait a max of 1s for the port to become available.
        let start_wait = Instant::now();
        while start_wait.elapsed().as_secs() < 1
            && !is_port_reachable(format!("localhost:{}", port))
        {
            sleep(Duration::from_millis(100));
        }

        let proto = if args.into_iter().any(|x| x == "--cert".into()) {
            "https".to_string()
        } else {
            "http".to_string()
        };
        let url = format!(
            "{proto}://{host}:{port}",
            proto = proto,
            host = host.to_string(),
            port = port
        );

        Ok(ProbyProcess {
            child,
            host,
            port,
            url,
        })
    }

    /// Generate a URL where we check ourselves for connectivity.
    pub fn selfcheck(&self) -> String {
        format!("{}/{}:{}", self.url, self.host, self.port)
    }
}
