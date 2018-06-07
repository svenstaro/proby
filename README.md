# proby [![Build Status](https://travis-ci.com/svenstaro/proby.svg?branch=master)](https://travis-ci.com/svenstaro/proby)
Probe whether hosts are reachable on certain ports and return result on HTTP

## What is this

This tool is a very simple web server that takes requests on HTTP to check
whether they are connectable on a provided port. It returns 200 by default if
the port is connectable and 400 if it isn't.

## Installation

Just grab one of the statically linked builds from the [Releases
page](https://github.com/svenstaro/proby/releases) and you're good to go!

## Building

You need a recent Rust nightly installed. It is recommended to use
[rustup](https://github.com/rust-lang-nursery/rustup.rs) for this.

Then just type

    cargo build --release

After the build, a binary will appear here: `target/release/proby`.

## Usage

Run the application using either

    cargo run --release

or

    target/release/proby

Example request for checking whether port 1337 is connectable on host example.com:

    curl localhost:8000/example.com:1337

This will return 200 if it is connectable and 400 if it isn't.

You can also use IPv4s or IPv6s, of course:

    curl localhost:8000/8.8.8.8:1337
    curl localhost:8000/2001:4860:4860::8888:1337

If you'd like to customize the return codes, you can do so by setting the
request parameters `good` and `bad` like so:

    curl localhost:8000/example.com:1337?good=201&bad=401

You can also configure a timeout (in seconds) using:

    curl localhost:8000/example.com:1337?timeout=2

The default timeout is one second.

To change the port this service listens on, specify the `ROCKET_PORT` environment
variable on launch:

    ROCKET_PORT=5555 cargo run --release
