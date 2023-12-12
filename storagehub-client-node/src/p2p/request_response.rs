use anyhow::anyhow;
use libp2p::request_response::{Event as RequestResponseEvent, Message};
use tracing::{debug, error, info, warn};

use crate::p2p::service::FileResponse;

use super::service::{FileRequest, Service};

impl Service {
	pub(crate) fn handle_req_res(
		&mut self,
		event: RequestResponseEvent<FileRequest, FileResponse>,
	) {
		match event {
			RequestResponseEvent::Message { peer, message } => match message {
				Message::Request { request, channel, .. } => {
					debug!(
                            "[RequestResponseEvent::Message::Request] - with request {:?} has been received from a peer {}.",
                            request,
                            peer
                        );

					let file = match std::fs::read(format!("{}/{}", self.file_path, request.0)) {
						Ok(file) => FileResponse(file),
						Err(e) => {
							error!("[RequestResponseEvent::Message::Request] - failed to read file: {:?}", e);
							return;
						},
					};

					if self
						.swarm
						.behaviour_mut()
						.request_response
						.send_response(channel, file)
						.is_err()
					{
						error!("[BehaviourEvent::RequestMessage] failed to send response")
					}

					info!(
						"[RequestResponseEvent::Message::Request] - sending FileResponse to peer {}.",
						peer
					);
				},
				Message::Response { request_id, response } => {
					debug!(
                        "[RequestResponseEvent::Message::Response] - with request_id {} and response {:?}.",
                        request_id,
                        response
                    );

					if let Some(request) = self.pending_responses.remove(&request_id) {
						if request.send(Ok(response.0)).is_err() {
							warn!("[RequestResponseMessage::Response] - failed to send request: {request_id:?}");
						}

						info!(
							"[RequestResponseEvent::Message::Response] - received FileResponse from peer {}.",
							peer
						);
					}
				},
			},
			RequestResponseEvent::ResponseSent { peer, request_id, .. } => {
				debug!(
					"[RequestResponseEvent::ResponseSent] - with peer {} and request_id {}.",
					peer, request_id
				);
			},
			RequestResponseEvent::InboundFailure { peer, request_id, error } => {
				error!(
                    "[RequestResponseEvent::InboundFailure] - with peer {} and request_id {} and error {:?}.",
                    peer,
                    request_id,
                    error
                );
			},
			RequestResponseEvent::OutboundFailure { request_id, error, .. } => {
				error!(
					"[RequestResponseEvent::OutboundFailure] - with request_id {} and error {:?}.",
					request_id, error
				);

				let _ = self
					.pending_responses
					.remove(&request_id)
					.expect("Request to still be pending.")
					.send(Err(anyhow!("Outbound failure.")));
			},
		}
	}
}
