use std::time::Duration;

use anyhow::Result;
use libp2p::{futures::StreamExt, swarm::SwarmEvent};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use tokio::{sync::oneshot, time::timeout};
use tracing::{error, info};

use crate::{lightclient, options, Role};

use super::service::Service;

fn setup_logger(level: LevelFilter) {
	if let Err(err) = SimpleLogger::new().with_level(level).with_utc_timestamps().init() {
		error!("Logger already set {:?}:", err)
	}
}

#[tokio::test]
async fn test_network_start() {
	setup_logger(LevelFilter::Info);

	let mut service = Service::new(Role::User, 23456, "./tmp/files-to-upload".to_string())
		.expect("Failed to create service");

	loop {
		if let SwarmEvent::NewListenAddr { address, .. } =
			timeout(Duration::from_secs(5), service.swarm.select_next_some())
				.await
				.expect("event to be received")
		{
			info!("SwarmEvent::NewListenAddr: {address:?}");
			break;
		}
	}
}

// #[tokio::test]
// async fn test_network_req_res() -> Result<()> {
// 	setup_logger(LevelFilter::Info);
// 	let mut config = NetworkConfig::default();

// 	let (mut node_1, node_1_addrs, peer_id_1, ..) = network_init(&mut config, None, None).await?;
// 	let (mut node_2, _, peer_id_2, ..) =
// 		network_init(&mut config, Some(node_1_addrs), None).await?;

// 	// Wait for at least one connection
// 	loop {
// 		if let SwarmEvent::ConnectionEstablished { peer_id, .. } =
// 			node_1.swarm.select_next_some().await
// 		{
// 			info!("[SwarmEvent::ConnectionEstablished]: {peer_id:?}, {peer_id_1:?}: ");
// 			break;
// 		}
// 	}

// 	let node_1_sender = node_1.command_sender();
// 	tokio::task::spawn(async move { node_1.start().await.unwrap() });

// 	let (sender, _) = oneshot::channel();
// 	let request = UrsaExchangeRequest(RequestType::CarRequest("Qm".to_string()));
// 	let msg = NetworkCommand::SendRequest {
// 		peer_id: peer_id_2,
// 		request: Box::new(request),
// 		channel: sender,
// 	};

// 	assert!(node_1_sender.send(msg).is_ok());

// 	loop {
// 		if let SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
// 			RequestResponseEvent::Message { peer, message },
// 		)) = timeout(Duration::from_secs(5), node_2.swarm.select_next_some())
// 			.await
// 			.expect("event to be received")
// 		{
// 			info!("[RequestResponseEvent::Message]: {peer:?}, {message:?}");
// 			break;
// 		}
// 	}

// 	Ok(())
// }
