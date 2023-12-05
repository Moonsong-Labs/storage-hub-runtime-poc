use clap::{Args, Parser, ValueEnum};
use tracing::Level;

use crate::runtimes::support::SupportedRuntime;

#[derive(Parser, Debug)]
pub(crate) struct Options {
	#[arg(help = "Sets the level of verbosity")]
	#[arg(long, default_value = "info")]
	pub(crate) log_level: Level,
	#[command(flatten)]
	#[command(next_help_heading = "libp2p Options")]
	pub(crate) libp2p_options: Libp2pOptions,
	#[command(flatten)]
	#[command(next_help_heading = "Light Client Options")]
	pub(crate) light_client_options: LightClientOptions,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Role {
	User,
	MspProvider,
	BspProvider,
}

#[derive(Args, Debug)]
pub(crate) struct Libp2pOptions {
	/// Fixed value to generate deterministic peer ID.
	#[clap(long)]
	pub(crate) secret_key_seed: Option<u8>,

	#[clap(long)]
	pub(crate) port: u16,
}

#[derive(Args, Debug, Clone)]
pub struct LightClientOptions {
	/// Determines whether to run the application as a specific storage provider or as a user
	#[arg(
		help = "Determines whether to run the application as a specific storage provider or as a user"
	)]
	#[arg(long, default_value = "msp-provider", value_enum)]
	pub run_as: Role,
	/// Chain to connect with.
	///
	/// This will automatically fill determine the ws address based on the selected chain.
	/// You can pass in the [`RunCommands::ws_address`] to override this.
	#[arg(help = "Chain to connect with")]
	#[arg(long, default_value = "local", value_enum)]
	pub chain: SupportedRuntime,
	/// Websocket address to connect to.
	///
	/// This will override the default ws address selected based on the chain.
	#[arg(help = "Websocket address to connect to")]
	#[arg(long)]
	pub ws_address: Option<String>,
	/// Path where files are uploaded from.
	///
	/// This is used by Users.
	#[arg(help = "Path where files are uploaded from")]
	#[arg(long, default_value = "./")]
	pub upload_path: String,
	/// Path where files will be download to.
	///
	/// This is used by Msp and Bsp providers.
	#[arg(help = "Path where files will be download to")]
	#[arg(long, default_value = "./")]
	pub download_path: String,
}
