// SPDX-License-Identifier: MIT OR Apache-2.0
use std::process::ExitCode;

use clap::Parser;
use imaguncula_cli::{Cli, execute};

fn main() -> ExitCode {
    match execute(Cli::parse()).and_then(|success| success.stdout()) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("imaguncula: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}
