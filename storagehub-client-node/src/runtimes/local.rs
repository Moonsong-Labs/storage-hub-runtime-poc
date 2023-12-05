#[subxt::subxt(
    runtime_metadata_path = "metadata/local.scale",
    derive_for_all_types = "Clone, PartialEq"
)]
mod node_runtime {}

use std::{fs, path::Path, str::FromStr, thread, time};

use libp2p::{multiaddr::Protocol, Multiaddr, PeerId};
use node_runtime::pallet_file_system::events::NewStorageRequest;
use subxt::utils::AccountId32;
use tracing::{debug, error, info};

use crate::{client::StorageHub, errors::StorageHubError};

pub(crate) async fn run_and_subscribe_to_events(
    storage_hub: &mut StorageHub,
) -> Result<(), StorageHubError> {
    info!("Subscribe 'NewStorageRequest' on-chain finalized event");

    let api = StorageHub::create_online_client_from_rpc(storage_hub.rpc_client.clone())
        .await
        .expect("Failed to create online client from rpc client");

    let mut block_sub = api.blocks().subscribe_finalized().await?;

    while let Some(block) = block_sub.next().await {
        let block = block?;
        debug!("Received block: {}", block.hash());

        let events = block.events().await?;

        // Event --> storage::NewStorageRequest
        if let Some(event) = events.find_first::<NewStorageRequest>()? {
            debug!("Received event storage::NewStorageRequest: {:?}", event);

            let account_id: AccountId32 = AccountId32::from_str(&event.who.to_string())
                .expect("Failed to convert `who` to AccountId32");

            let mut addr: Multiaddr = Multiaddr::from_str(
                String::from_utf8(event.sender_multiaddress.0)
                    .expect("Failed to cast event address bytes to Multiaddr")
                    .as_str(),
            )
            .expect("Failed to cast string to Multiaddr");

            let file_id: String = String::from_utf8(event.location.0.to_vec())
                .expect("Failed to convert bounded vec to string for file_id");
            let content_hash: String = event.fingerprint.to_string();
            let size: String = event.size.to_string();

            info!(
                "Received NewStorageRequest event - account_id: {}, peer: {}, file_id: {}, content_hash: {}, size: {}",
                account_id, addr, file_id, content_hash, size
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
                .request_file(peer_id, addr, file_id.clone())
                .await
            {
                Ok(file) => {
                    tracing::info!("Received file from peer {:?}", peer_id);

                    let file_path = format!("{}/{}", storage_hub.file_download_path, file_id);

                    // Create the directory if it does not exist
                    if !Path::new(&storage_hub.file_download_path).exists() {
                        fs::create_dir_all(&storage_hub.file_download_path)
                            .expect("Failed to create directory");
                    }

                    // Download the file to the specified location
                    fs::write(&file_path, &file).expect("Failed to write file");

                    info!("File downloaded to: {}", file_path);

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
