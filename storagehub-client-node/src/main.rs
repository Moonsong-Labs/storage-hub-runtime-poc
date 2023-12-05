use std::{collections::HashMap, error::Error};

use async_std::{io, task::spawn};
use clap::Parser;
use client::StorageHub;
use futures::{
    channel::{mpsc, oneshot},
    prelude::*,
};
use libp2p::{
    identify,
    multiaddr::Protocol,
    request_response::{self, OutboundRequestId},
    swarm::{NetworkBehaviour, SwarmEvent},
    Multiaddr, PeerId, Swarm,
};
use network::{Command, Event};
use serde::{Deserialize, Serialize};

use crate::config::Options;

mod client;
mod config;
mod errors;
mod network;
pub mod runtimes;

// We create a custom network behaviour that combines Kademlia, request_response and identify.
#[derive(NetworkBehaviour)]
struct Behaviour {
    request_response: request_response::cbor::Behaviour<FileRequest, FileResponse>,
    identify: identify::Behaviour,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FileRequest(String);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FileResponse(Vec<u8>);

type HashMapStore<K, R> = HashMap<K, oneshot::Sender<Result<R, Box<dyn Error + Send>>>>;

pub(crate) struct EventLoop {
    swarm: Swarm<Behaviour>,
    command_receiver: mpsc::Receiver<Command>,
    event_sender: mpsc::Sender<Event>,
    pending_dial: HashMapStore<PeerId, ()>,
    pending_request_file: HashMapStore<OutboundRequestId, Vec<u8>>,
}

impl EventLoop {
    fn new(
        swarm: Swarm<Behaviour>,
        command_receiver: mpsc::Receiver<Command>,
        event_sender: mpsc::Sender<Event>,
    ) -> Self {
        Self {
            swarm,
            command_receiver,
            event_sender,
            pending_dial: Default::default(),
            pending_request_file: Default::default(),
        }
    }

    /// Run the event loop.
    ///
    /// This function will listen on stdin for commands, on swarm events and on commands from the
    /// network client.
    pub(crate) async fn run(mut self) {
        let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();
        loop {
            futures::select! {
                line = stdin.select_next_some() => self.handle_input_line(line.expect("Stdin not to close")).await,
                event = self.swarm.next() => self.handle_event(event.expect("Swarm stream to be infinite.")).await,
                command = self.command_receiver.next() => match command {
                    Some(c) => self.handle_command(c).await,
                    // Command channel closed, thus shutting down the network event loop.
                    None=>  return,
                },
            }
        }
    }

    async fn handle_event(&mut self, event: SwarmEvent<BehaviourEvent>) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                tracing::info!(
                    "Multiaddr listening on {}/p2p/{}",
                    address,
                    self.swarm.local_peer_id()
                );
            }
            // Prints peer id identify info is being sent to.
            SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Sent {
                peer_id,
                ..
            })) => {
                tracing::info!("Identify sent to {:?}", peer_id);
            }
            // Prints out the info received via the identify event
            SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received {
                info,
                ..
            })) => {
                tracing::info!("Identify received: {:?}", info);
            }
            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                request_response::Event::Message { message, .. },
            )) => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    tracing::info!(request = ?request, "InboundRequest SwarmEvent");
                    self.event_sender
                        .send(Event::InboundRequest {
                            file_id: request.0,
                            channel,
                        })
                        .await
                        .expect("Event receiver not to be dropped.");
                }
                request_response::Message::Response {
                    request_id,
                    response,
                } => {
                    let _ = self
                        .pending_request_file
                        .remove(&request_id)
                        .expect("Request to still be pending.")
                        .send(Ok(response.0));
                }
            },
            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                request_response::Event::OutboundFailure {
                    request_id, error, ..
                },
            )) => {
                let _ = self
                    .pending_request_file
                    .remove(&request_id)
                    .expect("Request to still be pending.")
                    .send(Err(Box::new(error)));
            }
            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(
                request_response::Event::ResponseSent { .. },
            )) => {}
            SwarmEvent::IncomingConnection { .. } => {}
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                tracing::info!(peer=%peer_id, ?endpoint, "Established new connection");

                if endpoint.is_dialer() {
                    if let Some(_sender) = self.pending_dial.remove(&peer_id) {}
                }
            }
            SwarmEvent::ConnectionClosed { .. } => {}
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                if let Some(peer_id) = peer_id {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Err(Box::new(error)));
                    }
                }
            }
            SwarmEvent::IncomingConnectionError { .. } => {}
            SwarmEvent::Dialing {
                peer_id: Some(peer_id),
                ..
            } => eprintln!("Dialing {peer_id}"),
            SwarmEvent::NewExternalAddrCandidate { address } => {
                tracing::info!("New external address candidate {}", address);
            }
            e => panic!("{e:?}"),
        }
    }

    async fn handle_input_line(&mut self, line: String) {
        let mut args = line.split(' ');

        match args.next() {
            // This is considered to be the event that triggers the request for storage.
            // This will probably come from the light client that listens on events from the blockchain.
            Some("REQUEST_STORAGE") => {
                let file_id = match args.next() {
                    Some(c) => c.to_string(),
                    None => {
                        eprintln!("Expected file name");
                        return;
                    }
                };
                // this should be the peer id of the user that has the file
                let mut addr = match args.next() {
                    Some(addr) => addr.parse::<Multiaddr>().unwrap(),
                    None => {
                        eprintln!("Expected multiaddr including peer id");
                        return;
                    }
                };

                let peer_id: PeerId = match addr.pop().unwrap() {
                    Protocol::P2p(peer_id) => peer_id,
                    _ => {
                        eprintln!("Expected peer id in multiaddr");
                        return;
                    }
                };

                self.swarm
                    .behaviour_mut()
                    .request_response
                    .add_address(&peer_id, addr.clone());

                self.event_sender
                    .send(Event::OutboundRequest {
                        peer: peer_id,
                        addr,
                        file_id,
                    })
                    .await
                    .expect("Event receiver not to be dropped.");
            }
            _ => {
                eprintln!("Expected REQUEST_STORAGE");
            }
        }
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::RequestFile {
                file_id: file_name,
                peer,
                addr,
                sender,
            } => {
                self.swarm
                    .behaviour_mut()
                    .request_response
                    .add_address(&peer, addr.clone());
                let request_id = self
                    .swarm
                    .behaviour_mut()
                    .request_response
                    .send_request(&peer, FileRequest(file_name));
                self.pending_request_file.insert(request_id, sender);
            }
            Command::RespondFile { file, channel } => {
                self.swarm
                    .behaviour_mut()
                    .request_response
                    .send_response(channel, FileResponse(file))
                    .expect("Connection to peer to be still open.");
            }
            Command::RemoveAddress { peer, multiaddr } => {
                tracing::info!("Removing address {} from peer {}", multiaddr, peer);
                self.swarm
                    .behaviour_mut()
                    .request_response
                    .remove_address(&peer, &multiaddr);
            }
        }
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opts = Options::parse();
    let _ = tracing_subscriber::fmt()
        .with_env_filter(opts.log_level.to_string())
        .try_init();

    let (mut network_client, network_events, event_loop) = network::new(
        opts.libp2p_options.secret_key_seed,
        opts.libp2p_options.port,
        opts.light_client_options.clone().upload_path,
    )
    .await?;

    spawn(StorageHub::run(
        opts.light_client_options,
        network_client.clone(),
    ));
    spawn(event_loop.run());
    network_client.run(network_events).await;

    Ok(())
}
