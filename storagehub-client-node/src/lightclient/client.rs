use std::{collections::HashMap, fs, path::Path, thread, time};

use clap::ValueEnum;
use sp_core::crypto;
use subxt::{backend::rpc::RpcClient, rpc_params, Error, OnlineClient, PolkadotConfig};
use tracing::{error, info, warn};

use crate::{lightclient::support::ChainPrefix, options, swarming, Role};

use super::{errors::StorageHubError, local, support::SupportedRuntime};

#[derive(ValueEnum, Clone, Debug)]
pub(crate) enum DevAccounts {
	Alice,
	Bob,
	Charlie,
	Dave,
}

/// The light client is responsible for connecting to StorageHub and
/// subscribing to relevant events based on their [`Role`] and submitting
/// transactions to the chain.
///
/// The light client executes swarm based actions through the `command_sender` channel.
pub(crate) struct Client {
	/// The account used to sign transactions.
	pub(crate) account: DevAccounts,
	/// The runtime to use.
	pub(crate) runtime: SupportedRuntime,
	/// The RPC client.
	pub(crate) rpc_client: RpcClient,
	/// The mpsc channel to send commands to the swarm.
	pub(crate) command_sender: swarming::service::CommandSender,
	/// The path where the files are downloaded.
	pub(crate) download_path: String,
}

impl Client {
	/// Run the light client.
	pub(crate) async fn run(
		run_as: Role,
		config: options::LightClientOptions,
		command_sender: swarming::service::CommandSender,
		download_path: String,
	) {
		match run_as {
			Role::User => Self::_run_as_user().await,
			Role::BspProvider =>
				Self::run_as_bsp_provider(config, command_sender, download_path).await,
			Role::MspProvider => Self::_run_as_msp_provider().await,
		}
	}

	/// Create a new light client.
	pub(crate) async fn new(
		config: &options::LightClientOptions,
		command_sender: swarming::service::CommandSender,
		download_path: String,
	) -> Self {
		let ws_address = config.ws_address.clone().unwrap_or(config.chain.ws_address());

		let rpc_client = Self::create_rpc_client(ws_address.clone())
			.await
			.expect("Failed to create rpc client");

		let online_client = Self::create_online_client_from_rpc(rpc_client.clone())
			.await
			.expect("Failed to create online client from rpc client");
		let version = online_client.runtime_version();

		let chain: String = rpc_client
			.request("system_chain", rpc_params![])
			.await
			.expect("Failed to get system chain");

		let name: String = rpc_client
			.request("system_name", rpc_params![])
			.await
			.expect("Failed to get system chain");

		let properties: HashMap<String, String> = rpc_client
			.request("system_properties", rpc_params![])
			.await
			.expect("Failed to get system properties");

		// Display SS58 addresses based on the connected chain
		let chain_prefix: ChainPrefix = if let Some(ss58_format) = properties.get("ss58Format") {
			ss58_format.parse::<u16>().expect("Failed to parse ss58 format to u16")
		} else {
			0
		};

		crypto::set_default_ss58_version(crypto::Ss58AddressFormat::custom(chain_prefix));

		info!(
			"Connected to {} network using {} * Substrate node {} v{:?}",
			chain, ws_address, name, version
		);

		// Create the directory if it does not exist
		if !Path::new(&download_path).exists() {
			fs::create_dir_all(&download_path).expect("Failed to create directory");
		}

		Client {
			download_path,
			account: config.dev_account.clone(),
			runtime: config.chain,
			rpc_client,
			command_sender,
		}
	}

	async fn _run_as_user() {}

	async fn run_as_bsp_provider(
		config: options::LightClientOptions,
		command_sender: swarming::service::CommandSender,
		download_path: String,
	) {
		let mut n = 1_u32;
		loop {
			let mut client =
				Client::new(&config, command_sender.clone(), download_path.clone()).await;
			if let Err(e) = client.run_and_subscribe_to_events().await {
				match e {
					StorageHubError::SubscriptionFinished => warn!("{}", e),
					_ => {
						error!("{}", e);
						let sleep_min = u32::pow(3, n);
						info!("Sleeping for {}", sleep_min);
						thread::sleep(time::Duration::from_secs((60 * sleep_min).into()));
						n += 1;
						continue;
					},
				}
				thread::sleep(time::Duration::from_secs(1));
			}
		}
	}

	async fn _run_as_msp_provider() {
		unimplemented!("MSP provider execution not implemented.")
	}

	async fn run_and_subscribe_to_events(&mut self) -> Result<(), StorageHubError> {
		match self.runtime {
			SupportedRuntime::Local | SupportedRuntime::Compose => local::run(self).await,
		}
	}

	pub(crate) async fn create_rpc_client(ws_address: String) -> Result<RpcClient, Error> {
		RpcClient::from_url(ws_address).await
	}

	pub(crate) async fn create_online_client_from_rpc(
		rpc_client: RpcClient,
	) -> Result<OnlineClient<PolkadotConfig>, Error> {
		OnlineClient::from_rpc_client(rpc_client).await
	}
}
