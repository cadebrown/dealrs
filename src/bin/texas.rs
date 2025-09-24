//! Simple program to play Texas Holdem in a command-line interface (CLI) like your terminal, uses the [dealrs::texas::cli] module.

use clap::Parser;

use dealrs::texas::cli;

/// Entrypoint for the Texas Holdem CLI program, which just parses and runs the CLI from the provided arguments.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::Args::parse().run()
}
