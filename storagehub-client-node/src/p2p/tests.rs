#[tokio::test]
async fn test_network_start() {
	use std::time::Duration;

	use libp2p::{futures::StreamExt, swarm::SwarmEvent};
	use log::LevelFilter;
	use simple_logger::SimpleLogger;
	use tokio::time::timeout;
	use tracing::{error, info};

	use crate::Role;

	use super::service::Service;

	if let Err(err) = SimpleLogger::new().with_level(LevelFilter::Info).with_utc_timestamps().init()
	{
		error!("Logger already set {:?}:", err)
	}
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
