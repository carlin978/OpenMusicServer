use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Cli {
	#[command(subcommand)]
	pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
	///Show copyright info
	Copyright {
		///Show full license text
		#[arg(long)]
		full_license: bool,
	},
	///Start server
	Init {
		///Run server as a daemon - unimplemented
		#[arg(short, long)]
		daemon: bool,
	},
}
