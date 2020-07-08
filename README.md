# proby

[![GitHub Actions Workflow](https://github.com/svenstaro/proby/workflows/Build/badge.svg)](https://github.com/svenstaro/proby/actions)
[![Docker Cloud Build Status](https://img.shields.io/docker/cloud/build/svenstaro/proby)](https://cloud.docker.com/repository/docker/svenstaro/proby/)
[![AUR](https://img.shields.io/aur/version/proby.svg)](https://aur.archlinux.org/packages/proby/)
[![Crates.io](https://img.shields.io/crates/v/proby.svg)](https://crates.io/crates/proby)
[![license](http://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/svenstaro/proby/blob/master/LICENSE)
[![Lines of Code](https://tokei.rs/b1/github/svenstaro/proby)](https://github.com/svenstaro/proby)

**Check whether hosts are reachable on certain ports and return result on HTTP**

Its intended purpose is to be a bridge server for services that can only probe container or application health on HTTP. Oh, and it's just a single binary that works everywhere!

## What is this

This tool is a very simple web server that takes requests on HTTP to check
whether they are connectable on a provided port. It returns `200 OK` by default if
the port is connectable and `503 Service Unavailable` if it isn't.

## Installation

Just grab one of the statically linked builds from the [Releases
page](https://github.com/svenstaro/proby/releases) and you're good to go!

## Running

All you have to do to run proby is to just call it:

    proby

If you don't like the default interface and port of proby, you can change it like this:

    proby -i 127.0.0.1 -p 9000

## Usage

### Basic

This makes proby listen only on the local loopback interface at port 9000.

Example request for checking whether port 1337 is connectable on host example.com:

    curl localhost:8080/example:1337
    example:1337 is connectable

This will return 200 if it is connectable and 503 if it isn't.

You can also use IPv4s or IPv6s, of course:

    curl localhost:8080/8.8.8.8:1337
    curl localhost:8080/2001:4860:4860::8888:1337

### Advanced

If you'd like to customize the return codes, you can do so by setting the
request parameters `good` and `bad` like so:

    curl localhost:8080/example.com:1337?good=201&bad=401

You can also configure a timeout (in seconds) using:

    curl localhost:8080/example.com:1337?timeout=2

The default timeout is one second.

## CLI usage

    proby 1.0.1
    Sven-Hendrik Haase <svenstaro@gmail.com>
    Check whether hosts are reachable on certain ports and return result on HTTP

    USAGE:
        proby [FLAGS] [OPTIONS]

    FLAGS:
        -h, --help       Prints help information
        -q, --quiet      Be quiet (log nothing)
        -V, --version    Prints version information
        -v, --verbose    Be verbose (log data of incoming and outgoing requests). If given twice it will also log the body
                         data

    OPTIONS:
        -i, --interfaces <interfaces>...    Interface to bind to [default: 0.0.0.0]
        -p, --port <port>                   Port on which to listen [default: 8080]

## Building

You need a recent stable version of Rust and Cargo installed.

Then just type

    cargo build --release

After the build, a binary will appear here: `target/release/proby`.

## Releasing

This is mostly a note for me on how to release this thing:

- `cargo release --dry-run`
- `cargo release`
- Releases will automatically be deployed by Github Actions.
- Docker images will automatically be built by Docker Hub.
- Update AUR package.
