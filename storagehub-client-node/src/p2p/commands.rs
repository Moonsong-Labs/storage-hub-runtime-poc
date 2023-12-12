use anyhow::Error;
use libp2p::{Multiaddr, PeerId};
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use super::service::{FileRequest, Service};

/// Network commands that can be sent to the service asynchrously through an mpsc channel.
#[derive(Debug)]
pub enum NetworkCommand {
	/// Dial an external peer.
	ExternalDial { multiaddr: Multiaddr, channel: oneshot::Sender<Result<(), Error>> },
	/// Get the current list of multiaddresses we are listening on.
	Multiaddresses { channel: oneshot::Sender<Vec<Multiaddr>> },
	/// Request a file from a peer.
	RequestFile {
		file_id: String,
		peer_id: PeerId,
		multiaddr: Multiaddr,
		channel: oneshot::Sender<Result<Vec<u8>, Error>>,
	},
}

impl Service {
	/// Handles a command.
	///
	/// Commands can be sent from any asynchronous context within the application. They normally act
	/// as the bridge between the rest of the application and the swarm.
	///
	/// To send a command, use the [`Service::command_sender`].
	pub fn handle_command(&mut self, command: NetworkCommand) -> Result<(), anyhow::Error> {
		match command {
			NetworkCommand::ExternalDial { multiaddr, channel } => {
				self.swarm.dial(multiaddr)?;

				channel
					.send(Ok(()))
					.map_err(|_| anyhow::anyhow!("Failed to send dial command"))?;
			},
			NetworkCommand::Multiaddresses { channel } => {
				let multiaddresses: Vec<Multiaddr> =
					self.swarm.listeners().map(|addr| addr.clone()).collect();

				channel
					.send(multiaddresses)
					.map_err(|_| anyhow::anyhow!("Failed to send multiaddresses"))?;
			},
			NetworkCommand::RequestFile { file_id, peer_id, multiaddr, channel } => {
				let swarm = self.swarm.behaviour_mut();

				// TODO remove this and add address from the `IdentifyEvent::Received` event.
				swarm.request_response.add_address(&peer_id, multiaddr);
				let request_id =
					swarm.request_response.send_request(&peer_id, FileRequest(file_id));

				self.pending_responses.insert(request_id, channel);
			},
		}
		Ok(())
	}

	/// Returns a sender that can be used to send commands to the swarm.
	pub fn command_sender(&self) -> UnboundedSender<NetworkCommand> {
		self.command_sender.clone()
	}
}
