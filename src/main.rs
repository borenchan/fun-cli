mod cli;
mod error;
mod impls;
mod ui;
mod utils;

use crate::cli::FunCli;
use clap::Parser;

fn main() {
    let cli = FunCli::parse();
    let commands = cli.command;
    commands.run();
}
