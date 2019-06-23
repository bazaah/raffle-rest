# Raffle REST API

This repository is an example REST API created for a Poppulo home test. It is built in pure rust, according to the goals laid out in the goals.md.

## Installation

From repo to running in 3 steps!

1. `git clone https://github.com/bazaah/raffle-rest.git ; cd raffle-rest`
2. `rustup default nightly`
3. `ROCKET_ENV=development cargo run --release`

*NOTE: you must have the rust toolchain installed, grab it for your OS [here](https://www.rust-lang.org/tools/install)*

### Usage

Using the API is simple: navigate to address set in the `Rocket.toml` entry that corresponds to your `ROCKET_ENV`, either in your web browser or through another http reader (`curl`, `postman`, etc), and start running commands!

You can find the complete API spec in assets/api, but here are a few examples:

`/ticket`
`/ticket/list`
`/eval/<id>`

### License

**MIT**
