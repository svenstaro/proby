# Changelog

<!-- next-header -->

## [Unreleased] - ReleaseDate

# [1.0.0] - 2020-07-05

- Start using `cargo release` for release management.
- Bump deps.
- Removed badges from `Cargo.toml`. They're being phased out.

# [0.4.0] - 2020-01-25

- Add --quiet flag to mute all stdout output.
- Add --verbose flag to print all incoming connections and what their queried host.

# [0.3.0] - 2020-01-24

- Switch from Rocket to actix-web to make this work on stable Rust.
