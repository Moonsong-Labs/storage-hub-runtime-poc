use crate::Role;
use anyhow::{anyhow, Result};
use libp2p::{
	futures::StreamExt, request_response::OutboundRequestId, swarm::NetworkBehaviour, Swarm,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, time::Duration};
use tokio::{
	select,
	sync::{
		mpsc::{UnboundedReceiver, UnboundedSender},
		oneshot::Sender,
	},
};
use tracing::info;

use lazy_static::lazy_static;
use libp2p::{
	identify,
	request_response::{self, ProtocolSupport},
	StreamProtocol,
};
use tokio::sync::mpsc::unbounded_channel;

use super::commands::NetworkCommand;

/// Defines max_negotiating_inbound_streams constant for the swarm.
/// It must be set for large plots.
const SWARM_MAX_NEGOTIATING_INBOUND_STREAMS: usize = 100000;
/// How long will connection be allowed to be open without any usage
const IDLE_CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);

pub const IDENTIFY_PROTOCOL: &str = "/storagehub/id/0.0.1";
pub const REQUEST_RESPONSE_PROTOCOL: &str = "/storagehub/req-res/0.0.1";

lazy_static! {
	pub(crate) static ref REQUEST_RESPONSE_PROTOCOL_NAME: StreamProtocol =
		StreamProtocol::new(REQUEST_RESPONSE_PROTOCOL);
}

pub(crate) type Port = u16;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FileRequest(pub(crate) String);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FileResponse(pub(crate) Vec<u8>);

/// Custom network behaviour that combines request_response and identify.
///
/// Checkout a more in depth [macro explanation](https://docs.rs/libp2p/latest/libp2p/swarm/trait.NetworkBehaviour.html#custom-networkbehaviour-with-the-derive-macro)
#[derive(NetworkBehaviour)]
pub(crate) struct Behaviour {
	identify: identify::Behaviour,
	pub(crate) request_response: request_response::cbor::Behaviour<FileRequest, FileResponse>,
}

type HashMapStore<K, R> = HashMap<K, Sender<Result<R, anyhow::Error>>>;

pub(crate) type CommandSender = UnboundedSender<NetworkCommand>;

pub(crate) struct Service {
	/// Swarm drives both the `Transport` and `NetworkBehaviour` forward, passing commands from the
	/// `NetworkBehaviour` to the `Transport` as well as events from the Transport to the
	/// `NetworkBehaviour`
	pub(crate) swarm: Swarm<Behaviour>,
	/// Handles outbound messages to peers.
	pub(crate) command_sender: CommandSender,
	/// Handles inbound messages from peers.
	pub(crate) command_receiver: UnboundedReceiver<NetworkCommand>,
	/// Pending `request_response` requests.
	pub(crate) pending_responses: HashMapStore<OutboundRequestId, Vec<u8>>,
	/// Path to the file to be sent.
	pub(crate) file_path: String,
}

impl Service {
	/// Start the client node service loop.
	///
	/// Poll `swarm` and `command_receiver` from [`Service`].
	/// - `swarm` handles the network events [`SwarmEvent`](libp2p::swarm::SwarmEvent).
	/// - `command_receiver` handles inbound commands [`NetworkCommand`].
	pub async fn run(mut self) -> Result<()> {
		info!("Node starting up with peerId {:?}", self.swarm.local_peer_id());
		loop {
			select! {
				event = self.swarm.next() => {
					let event = event.ok_or_else(|| anyhow!("Event invalid!"))?;
					self.handle_swarm_event(event).await;
				},
				command = self.command_receiver.recv() => {
					let command = command.ok_or_else(|| anyhow!("Command invalid!"))?;
					self.handle_command(command).expect("Handle rpc command.");
				},
			}
		}
	}

	/// Creates a new swarm with the given `role` and `port`.
	///
	/// Based on the StorageHub role specification, the swarm will be configured to support the
	/// required protocols for the given role (i.e. users and storage providers will need only a
	/// specific set of protocols enabled). (TODO: link to the role specification)
	pub(crate) fn new(
		role: Role,
		port: Port,
		file_path: String,
	) -> Result<Service, Box<dyn Error>> {
		let mut swarm = libp2p::SwarmBuilder::with_new_identity()
			.with_tokio()
			// Transport protocol: https://docs.rs/libp2p/latest/libp2p/trait.Transport.html
			.with_tcp(
				libp2p::tcp::Config::default(),
				libp2p::tls::Config::new,
				libp2p::yamux::Config::default,
			)?
			// NetworkBehaviours: https://docs.rs/libp2p/latest/libp2p/swarm/trait.NetworkBehaviour.html#
			.with_behaviour(|key| {
				Ok(Behaviour {
					identify: identify::Behaviour::new(identify::Config::new(
						IDENTIFY_PROTOCOL.into(),
						key.public(),
					)),
					request_response: request_response::cbor::Behaviour::new(
						[(
							StreamProtocol::new(REQUEST_RESPONSE_PROTOCOL),
							// TODO figure out outbound and inbound requirements for each role
							match role {
								Role::User => ProtocolSupport::Full,
								Role::BspProvider => ProtocolSupport::Full,
								Role::MspProvider => ProtocolSupport::Full,
							},
						)],
						request_response::Config::default(),
					),
				})
			})?
			.with_swarm_config(|config| {
				config
					.with_max_negotiating_inbound_streams(SWARM_MAX_NEGOTIATING_INBOUND_STREAMS)
					.with_idle_connection_timeout(IDLE_CONNECTION_TIMEOUT)
			})
			.build();

		// Tell the swarm to listen on all interfaces and a random, OS-assigned
		// port.
		swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", port).parse()?)?;

		// TODO with unbounded_channel, limits will need to be set to avoid running out of memory
		let (command_sender, command_receiver) = unbounded_channel();

		// check if file path exists, if not create it
		if !std::path::Path::new(&file_path).exists() {
			std::fs::create_dir_all(&file_path)?;
		}

		Ok(Service {
			swarm,
			command_sender,
			command_receiver,
			pending_responses: HashMap::new(),
			file_path,
		})
	}
}
