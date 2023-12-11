use std::{collections::HashMap, thread, time};

use crate::{
    config::DevAccounts,
    network::{self},
};
use sp_core::crypto;
use subxt::{backend::rpc::RpcClient, rpc_params, Error, OnlineClient, PolkadotConfig};
use tracing::{error, info, warn};

use crate::{
    config::{LightClientOptions, Role},
    errors::StorageHubError,
    runtimes::{
        local,
        support::{ChainPrefix, SupportedRuntime},
    },
};

pub(crate) struct StorageHub {
    pub(crate) file_download_path: String,
    pub(crate) account: DevAccounts,
    pub(crate) runtime: SupportedRuntime,
    pub(crate) rpc_client: RpcClient,
    pub(crate) network_client: network::Client,
}

impl StorageHub {
    pub(crate) async fn new(config: &LightClientOptions, network_client: network::Client) -> Self {
        let ws_address = config
            .ws_address
            .clone()
            .unwrap_or(config.chain.ws_address());

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
            ss58_format
                .parse::<u16>()
                .expect("Failed to parse ss58 format to u16")
        } else {
            0
        };

        crypto::set_default_ss58_version(crypto::Ss58AddressFormat::custom(chain_prefix));

        info!(
            "Connected to {} network using {} * Substrate node {} v{:?}",
            chain, ws_address, name, version
        );
        StorageHub {
            file_download_path: config.download_path.clone(),
            account: config.dev_account.clone(),
            runtime: SupportedRuntime::Local,
            rpc_client,
            network_client,
        }
    }

    pub(crate) async fn run(config: LightClientOptions, network_client: network::Client) {
        match config.run_as {
            Role::User => Self::_run_as_user().await,
            Role::BspProvider => Self::run_as_bsp_provider(config, network_client).await,
            Role::MspProvider => Self::_run_as_msp_provider().await,
        }
    }

    async fn _run_as_user() {}

    async fn run_as_bsp_provider(config: LightClientOptions, network_client: network::Client) {
        let mut n = 1_u32;
        loop {
            let mut msp = StorageHub::new(&config, network_client.clone()).await;
            if let Err(e) = msp.run_and_subscribe_to_events().await {
                match e {
                    StorageHubError::SubscriptionFinished => warn!("{}", e),
                    _ => {
                        error!("{}", e);
                        let sleep_min = u32::pow(3, n);
                        info!("Sleeping for {}", sleep_min);
                        thread::sleep(time::Duration::from_secs((60 * sleep_min).into()));
                        n += 1;
                        continue;
                    }
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
            SupportedRuntime::Local | SupportedRuntime::Compose => {
                local::run_and_subscribe_to_events(self).await
            }
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
