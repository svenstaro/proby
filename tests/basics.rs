mod utils;

use assert_cmd::prelude::*;
use std::io::Read;
use std::process::Command;
use structopt::clap::{crate_name, crate_version};
use surf;

use utils::{Error, ProbyProcess};

/// Show help and exit.
#[test]
fn help_shows() -> Result<(), Error> {
    Command::cargo_bin("proby")?
        .arg("--help")
        .assert()
        .success();

    Ok(())
}

/// Show version and exit.
#[test]
fn version_shows() -> Result<(), Error> {
    Command::cargo_bin("proby")?
        .arg("-V")
        .assert()
        .success()
        .stdout(format!("{} {}\n", crate_name!(), crate_version!()));

    Ok(())
}

/// If provided with no options, we're shown some basic information on stdout.
#[actix_rt::test]
async fn has_some_output_by_default() -> Result<(), Error> {
    let mut dh = ProbyProcess::new(Vec::<String>::new())?;

    let name_and_version = format!("{} {}\n", crate_name!(), crate_version!());

    let resp_body = surf::get(&dh.url).recv_string().await?;
    assert!(resp_body.contains(&name_and_version));

    dh.child.kill()?;
    let mut output = String::new();
    dh.child
        .stdout
        .as_mut()
        .unwrap()
        .read_to_string(&mut output)?;

    assert!(output.contains(&name_and_version));

    Ok(())
}

/// If we pass --quiet, we get no output.
#[actix_rt::test]
async fn has_quiet_output() -> Result<(), Error> {
    let mut dh = ProbyProcess::new(vec!["--quiet"])?;

    println!("{}", dh.selfcheck());
    assert_eq!(surf::get(&dh.selfcheck()).await?.status(), 200);

    dh.child.kill()?;
    let mut output = String::new();
    dh.child
        .stdout
        .as_mut()
        .unwrap()
        .read_to_string(&mut output)?;

    assert!(output.is_empty());

    Ok(())
}

/// If we pass --verbose, we see every single connection.
#[actix_rt::test]
async fn has_verbose_output() -> Result<(), Error> {
    let mut dh = ProbyProcess::new(vec!["--verbose"])?;

    assert_eq!(surf::get(&dh.selfcheck()).await?.status(), 200);

    dh.child.kill()?;
    let mut output = String::new();
    dh.child
        .stdout
        .as_mut()
        .unwrap()
        .read_to_string(&mut output)?;

    assert!(output.contains("requesting check of"));

    Ok(())
}
