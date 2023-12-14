#[subxt::subxt(
	runtime_metadata_path = "metadata/local.scale",
	derive_for_all_types = "Clone, PartialEq"
)]
mod node_runtime {}

use libp2p::{multiaddr::Protocol, Multiaddr, PeerId};
use node_runtime::pallet_file_system::events::NewStorageRequest;
use std::{fs, str::FromStr, thread, time};
use subxt::{
	ext::sp_core::{sr25519::Pair, Pair as PairT},
	tx::PairSigner,
	utils::AccountId32,
};
use tokio::sync::oneshot;
use tracing::{debug, error, info};

use crate::{lightclient::client::DevAccounts, p2p};

use super::{client::Client, errors::StorageHubError};

pub(crate) async fn run(storage_hub: &mut Client) -> Result<(), StorageHubError> {
	info!("Subscribe 'NewStorageRequest' on-chain finalized event");

	let api = Client::create_online_client_from_rpc(storage_hub.rpc_client.clone())
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

			let mut sender_multiaddr: Multiaddr = Multiaddr::from_str(
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
                account_id, sender_multiaddr, file_id, content_hash, size
            );

			let (sender, receiver) = oneshot::channel();

			storage_hub
				.command_sender
				.send(p2p::commands::NetworkCommand::Multiaddresses { channel: sender })
				.expect("Failed to send get multiaddresses command");
			let multiaddresses = receiver.await.expect("Failed to receive multiaddresses");

			// find first multiaddr that is not localhost
			let multiaddr = multiaddresses
				.iter()
				.find(|multiaddr| {
					multiaddr.iter().any(|protocol| match protocol {
						Protocol::Ip4(ip) => ip != std::net::Ipv4Addr::new(127, 0, 0, 1),
						_ => false,
					})
				})
				.expect("Failed to find multiaddr that is not localhost");

			let peer = node_runtime::runtime_types::bounded_collections::bounded_vec::BoundedVec(
				multiaddr.as_ref().to_vec(),
			);

			let volunteer_tx = node_runtime::tx().pallet_file_system().bsp_volunteer(
				event.location,
				event.fingerprint,
				peer,
			);

			let account = match storage_hub.account {
				DevAccounts::Alice => "//Alice",
				DevAccounts::Bob => "//Bob",
				DevAccounts::Charlie => "//Charlie",
				DevAccounts::Dave => "//Dave",
			};
			let owner: Pair =
				Pair::from_string(account, None).expect("Failed to create pair from string");

			let signer = PairSigner::new(owner);
			let _ = api
				.tx()
				.sign_and_submit_then_watch_default(&volunteer_tx, &signer)
				.await?
				.wait_for_finalized_success()
				.await?;

			info!("Successfully volunteered for file_id: {}", file_id);

			let sender_peer_id: PeerId = match sender_multiaddr.pop().unwrap() {
				Protocol::P2p(peer_id) => peer_id,
				_ => {
					eprintln!("Expected peer id in multiaddr");
					continue;
				},
			};

			let (sender, receiver) = oneshot::channel();

			storage_hub
				.command_sender
				.send(p2p::commands::NetworkCommand::ExternalDial {
					multiaddr: sender_multiaddr.clone(),
					channel: sender,
				})
				.expect("Failed to send dial command");
			let _ = receiver.await.expect("Failed to receive dial command");

			let (sender, receiver) = oneshot::channel();

			storage_hub
				.command_sender
				.send(p2p::commands::NetworkCommand::RequestFile {
					file_id: file_id.clone(),
					peer_id: sender_peer_id,
					multiaddr: sender_multiaddr,
					channel: sender,
				})
				.expect("Failed to send request file command");
			let maybe_file = receiver.await.expect("Failed to receive file");

			match maybe_file {
				Ok(file) => {
					info!("Received file from peer {:?}", sender_peer_id);

					let file_path = format!("{}/{}", storage_hub.download_path, file_id);

					// Download the file to the specified location
					fs::write(&file_path, &file).expect("Failed to write file");

					info!("File downloaded to: {}", file_path);

					let wait: u64 = 3;
					info!("Waiting {} seconds before run batch", wait);
					thread::sleep(time::Duration::from_secs(wait));
				},
				Err(e) => {
					error!("Failed to request file: {}", e);
				},
			}
		}
	}
	// If subscription has closed for some reason await and subscribe again
	Err(StorageHubError::SubscriptionFinished)
}
