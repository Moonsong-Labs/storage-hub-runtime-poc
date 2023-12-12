use libp2p::identify::Event as IdentifyEvent;
use tracing::debug;

use crate::swarming::service::REQUEST_RESPONSE_PROTOCOL_NAME;

use super::service::Service;

impl Service {
	pub(crate) fn handle_identify(&mut self, identify_event: IdentifyEvent) {
		match identify_event {
			IdentifyEvent::Received { peer_id, info } => {
				debug!(
					"[IdentifyEvent::Received] - with version {} has been received from a peer {}.",
					info.protocol_version, peer_id
				);

				// add the peer to the request_response protocol
				if info
					.protocols
					.iter()
					.any(|name| name == &REQUEST_RESPONSE_PROTOCOL_NAME.clone())
				{
					let behaviour = self.swarm.behaviour_mut();

					for address in info.listen_addrs {
						debug!(
							"[IdentifyEvent::Received] - adding peer {} to request_response protocol with address {}",
							peer_id,
							address
						);
						behaviour.request_response.add_address(&peer_id, address);
					}
				}
			},
			IdentifyEvent::Sent { peer_id } => {
				debug!("[IdentifyEvent::Sent] - to peer {}.", peer_id);
			},
			IdentifyEvent::Pushed { peer_id, info } => {
				debug!("[IdentifyEvent::Pushed] - to peer {} with info {:?}.", peer_id, info);
			},
			IdentifyEvent::Error { peer_id, error } => {
				debug!("[IdentifyEvent::Error] - with peer {} and error {:?}.", peer_id, error);
			},
		}
	}
}
