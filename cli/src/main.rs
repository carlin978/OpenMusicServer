use anyhow::Context;
use clap::Parser;
use oms_types::config;

mod cli;

use cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
	let cli = Cli::parse();

	match cli.command {
		Commands::Copyright { full_license } => {
			if full_license {
				println!("{}", include_str!("../../LICENSE"));
			} else {
				println!("Copyright © 2025 carlin978");
			}
		}
		Commands::Init { daemon } => {
			let config = config::load_config().context("Failed to load config")?;

			if daemon {
				todo!("Daemon mode is not yet implemented")
			} else {
				oms_server::init(config).context("Failed to initialize server")?;
			}
		}
	}

	Ok(())
}
