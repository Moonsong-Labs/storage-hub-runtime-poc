use std::{error::Error, io::Write, time::Duration};

use futures::{
    channel::{mpsc, oneshot},
    prelude::*,
    SinkExt, Stream,
};
use libp2p::{
    identify, identity, noise,
    request_response::{self, ProtocolSupport, ResponseChannel},
    tcp, yamux, Multiaddr, PeerId, StreamProtocol,
};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use crate::{Behaviour, EventLoop, FileResponse};

/// Defines max_negotiating_inbound_streams constant for the swarm.
/// It must be set for large plots.
const SWARM_MAX_NEGOTIATING_INBOUND_STREAMS: usize = 100000;
/// How long will connection be allowed to be open without any usage
const IDLE_CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);

/// Create a new Swarm network
pub(crate) async fn new(
    secret_key_seed: Option<u8>,
    port: u16,
    upload_path: String,
) -> Result<(Client, impl Stream<Item = Event>, EventLoop), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let id_keys = match secret_key_seed {
        Some(seed) => {
            let mut bytes = [0u8; 32];
            bytes[0] = seed;
            identity::Keypair::ed25519_from_bytes(bytes).unwrap()
        }
        None => identity::Keypair::generate_ed25519(),
    };

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(id_keys)
        .with_async_std()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            Ok(Behaviour {
                identify: identify::Behaviour::new(identify::Config::new(
                    "/storagehub/0.1.0".to_string(),
                    key.public(),
                )),
                request_response: request_response::cbor::Behaviour::new(
                    [(
                        StreamProtocol::new("/storagehub/0.1.0"),
                        ProtocolSupport::Full,
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

    swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", port).parse()?)?;

    let (command_sender, command_receiver) = mpsc::channel(0);
    let (event_sender, event_receiver) = mpsc::channel(0);

    Ok((
        Client {
            upload_path,
            sender: command_sender,
            multiaddr: swarm.local_peer_id().clone(),
        },
        event_receiver,
        EventLoop::new(swarm, command_receiver, event_sender),
    ))
}

#[derive(Clone)]
pub(crate) struct Client {
    pub(crate) upload_path: String,
    pub(crate) sender: mpsc::Sender<Command>,
    pub(crate) multiaddr: PeerId,
}

impl Client {
    pub(crate) async fn run(
        &mut self,
        mut network_events: impl Stream<Item = Event> + std::marker::Unpin,
    ) {
        loop {
            match network_events.next().await {
                // Reply with the content of the file on incoming requests.
                Some(Event::InboundRequest { file_id, channel }) => {
                    info!(file_id = ?file_id, "InboundRequest network event");
                    self.respond_file(
                        std::fs::read(format!("{}/{}", self.upload_path, file_id)).unwrap(),
                        channel,
                    )
                    .await;

                    info!("Responded to inbound request for file {}", file_id);
                }
                Some(Event::OutboundRequest {
                    peer,
                    addr,
                    file_id,
                }) => {
                    info!(
                        "Sending request for file {} readme to peer {:?}",
                        file_id, peer
                    );

                    match self.request_file(peer, addr.clone(), file_id).await {
                        Ok(file) => {
                            tracing::info!("Received file from peer {:?}", peer);
                            std::io::stdout()
                                .write_all(&file)
                                .expect("Stdout to be open.");
                        }
                        Err(e) => {
                            error!("Failed to receive file from peer {:?}: {:?}", peer, e)
                        }
                    }

                    self.remove_address(&peer, &addr).await;
                }
                e => todo!("{:?}", e),
            }
        }
    }
    /// Request the content of the given file from the given peer.
    pub(crate) async fn request_file(
        &mut self,
        peer: PeerId,
        addr: Multiaddr,
        file_id: String,
    ) -> Result<Vec<u8>, Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::RequestFile {
                file_id,
                peer,
                addr,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not be dropped.")
    }

    /// Respond with the provided file content to the given request.
    pub(crate) async fn respond_file(
        &mut self,
        file: Vec<u8>,
        channel: ResponseChannel<FileResponse>,
    ) {
        self.sender
            .send(Command::RespondFile { file, channel })
            .await
            .expect("Command receiver not to be dropped.");
    }

    /// Remove request_response address
    pub(crate) async fn remove_address(&mut self, peer: &PeerId, multiaddr: &Multiaddr) {
        self.sender
            .send(Command::RemoveAddress {
                peer: peer.clone(),
                multiaddr: multiaddr.clone(),
            })
            .await
            .expect("Command receiver not to be dropped.");
    }
}

#[derive(Debug)]
pub enum Command {
    RequestFile {
        file_id: String,
        peer: PeerId,
        addr: Multiaddr,
        sender: oneshot::Sender<Result<Vec<u8>, Box<dyn Error + Send>>>,
    },
    RespondFile {
        file: Vec<u8>,
        channel: ResponseChannel<FileResponse>,
    },
    RemoveAddress {
        peer: PeerId,
        multiaddr: Multiaddr,
    },
}

#[derive(Debug)]
pub(crate) enum Event {
    /// An inbound request for a file.
    InboundRequest {
        file_id: String,
        channel: ResponseChannel<FileResponse>,
    },
    /// Send request for file to peer.
    OutboundRequest {
        peer: PeerId,
        addr: Multiaddr,
        file_id: String,
    },
}
