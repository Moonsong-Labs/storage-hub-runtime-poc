#[subxt::subxt(
    runtime_metadata_path = "metadata/local.scale",
    derive_for_all_types = "Clone, PartialEq"
)]
mod node_runtime {}

use std::{io::Write, str::FromStr, thread, time};

use libp2p::{multiaddr::Protocol, Multiaddr, PeerId};
use node_runtime::template_module::events::RequestStore;
use subxt::utils::AccountId32;
use tracing::{debug, error, info};

use crate::{client::StorageHub, errors::StorageHubError};

pub(crate) async fn run_and_subscribe_to_events(
    storage_hub: &mut StorageHub,
) -> Result<(), StorageHubError> {
    info!("Subscribe 'RequestStore' on-chain finalized event");

    let api = StorageHub::create_online_client_from_rpc(storage_hub.rpc_client.clone())
        .await
        .expect("Failed to create online client from rpc client");

    let mut block_sub = api.blocks().subscribe_finalized().await?;

    while let Some(block) = block_sub.next().await {
        let block = block?;
        debug!("Received block: {}", block.hash());

        let events = block.events().await?;

        // Event --> storage::RequestStore
        if let Some(event) = events.find_first::<RequestStore>()? {
            debug!("Received event storage::RequestStore: {:?}", event);

            let account_id: AccountId32 = AccountId32::from_str(&event.who.to_string())
                .expect("Failed to convert `who` to AccountId32");

            let mut addr: Multiaddr = Multiaddr::from_str(
                String::from_utf8(event.address.0)
                    .expect("Failed to cast event address bytes to Multiaddr")
                    .as_str(),
            )
            .expect("Failed to cast string to Multiaddr");

            let file_id: String = String::from_utf8(event.file.id.0)
                .expect("Failed to convert bounded vec to string for file_id");
            let content_hash: String = event.file.content_hash.to_string();

            info!(
                "Received RequestStore event - account_id: {}, peer: {}, file_id: {}, content_hash: {}",
                account_id, addr, file_id, content_hash
            );

            let peer_id: PeerId = match addr.pop().unwrap() {
                Protocol::P2p(peer_id) => peer_id,
                _ => {
                    eprintln!("Expected peer id in multiaddr");
                    continue;
                }
            };

            match storage_hub
                .network_client
                .request_file(peer_id, addr, file_id)
                .await
            {
                Ok(file) => {
                    tracing::info!("Received file from peer {:?}", peer_id);
                    std::io::stdout()
                        .write_all(&file)
                        .expect("Stdout to be open.");

                    let wait: u64 = 3;
                    info!("Waiting {} seconds before run batch", wait);
                    thread::sleep(time::Duration::from_secs(wait));
                }
                Err(e) => {
                    error!("Failed to request file: {}", e);
                }
            }
        }
    }
    // If subscription has closed for some reason await and subscribe again
    Err(StorageHubError::SubscriptionFinished)
}
