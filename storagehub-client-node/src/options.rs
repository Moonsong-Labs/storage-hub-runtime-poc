use clap::{Args, Parser};

use crate::{
	lightclient::{client::DevAccounts, support::SupportedRuntime},
	swarming, Role,
};

#[derive(Parser, Debug)]
pub(crate) struct Options {
	/// Determines whether to run the application as a specific storage provider or as a user
	#[arg(help = "Runs as a specific storage provider or as a user")]
	#[arg(long, value_enum)]
	pub run_as: Role,
	#[command(flatten)]
	#[command(next_help_heading = "libp2p Options")]
	pub(crate) libp2p_options: Libp2pOptions,
	#[command(flatten)]
	#[command(next_help_heading = "Light Client Options")]
	pub(crate) light_client_options: LightClientOptions,
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

#[derive(Args, Debug)]
pub(crate) struct Libp2pOptions {
	#[clap(long)]
	pub(crate) port: swarming::service::Port,
}

#[derive(Args, Debug, Clone)]
pub struct LightClientOptions {
	/// Chain to connect with.
	///
	/// This will automatically fill determine the ws address based on the selected chain.
	/// You can pass in the [`RunCommands::ws_address`] to override this.
	#[arg(help = "Chain to connect with")]
	#[arg(long, default_value = "local", value_enum)]
	pub chain: SupportedRuntime,
	/// Dev account to sign transactions with.
	#[arg(help = "Dev account to sign transactions with")]
	#[arg(long, default_value = "alice", value_enum)]
	pub dev_account: DevAccounts,
	/// Websocket address to connect to.
	///
	/// This will override the default ws address selected based on the chain.
	#[arg(help = "Websocket address to connect to")]
	#[arg(long)]
	pub ws_address: Option<String>,
}
