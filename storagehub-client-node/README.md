# PoC: Main Storage Provider

> This is a proof of concept and is by no means a finished product.

## Features

- [ ] `subxt` (light client)
  - [x] Connect to substrate node
  - [x] Subscribe to events
  - [x] Send extrinsics
  - [x] BSP node dispatches `bsp_volunteer` extrinsic prior to sending file request
  - [ ] Receive and queue multiple `NewStorageRequest`s events (right now it only can process one at a time)
- [ ] `libp2p` (peer-to-peer networking)
  - [x] BSP node sends file request
  - [x] Establish connection between User and BSP node
  - [x] Send file data
  - [x] Receive file data
  - [x] Multiple file requests for the same file from multiple BSP nodes
  - [ ] User validates BSP node is registered on chain
  - [ ] Add external address using `Identify` Behaviour (right now it only adds the address in the request_response `FileRequest` event)
  - [ ] MSP/BSP node validates data received from user node against `content_hash` from `NewStorageRequest` event

## How to run locally

### Build and run the substrate node

In one terminal run the storagehub runtime:

```bash
# project-root/storagehub-runtim
cargo build --release

./target/release/node-template --dev
```

### Run BSP node

```bash
RUST_LOG=info cargo run -- --run-as bsp-provider --chain local --port 35436 --dev-account alice --download-path "./tmp/downloaded-files/alice"
```

This will connect to the substrate node template via `subxt` (which utilizes `smoldot` in the background) and will subscribe to events being triggered by the substrate node.

It will specifically listen to the `NewStorageRequest` event which contains all the information required for the BSP node to send a file request to the user's peer address.

Output (truncated):

```bash
2023-12-12T20:54:21.878740Z  INFO libp2p_swarm: local_peer_id=12D3KooWSvD9mjiZsCxwH5zkJBTUELZYQ7qxpRw7NRYt8212GXWD
2023-12-12T20:54:21.879046Z  INFO storagehub_client::p2p::service: Node starting up with peerId PeerId("12D3KooWSvD9mjiZsCxwH5zkJBTUELZYQ7qxpRw7NRYt8212GXWD")
2023-12-12T20:54:21.879298Z  INFO storagehub_client::p2p::swarm: [SwarmEvent::NewListenAddr] - listen address: /ip4/127.0.0.1/tcp/35436/p2p/12D3KooWSvD9mjiZsCxwH5zkJBTUELZYQ7qxpRw7NRYt8212GXWD
2023-12-12T20:54:21.879362Z  INFO storagehub_client::p2p::swarm: [SwarmEvent::NewListenAddr] - listen address: /ip4/172.28.164.193/tcp/35436/p2p/12D3KooWSvD9mjiZsCxwH5zkJBTUELZYQ7qxpRw7NRYt8212GXWD
2023-12-12T20:54:21.894297Z  INFO storagehub_client::lightclient::client: Connected to Development network using ws://127.0.0.1:9944 * Substrate node Substrate Node vRuntimeVersion { spec_version: 100, transaction_version: 1 }
2023-12-12T20:54:21.894474Z  INFO storagehub_client::lightclient::local: Subscribe 'NewStorageRequest' on-chain finalized event
```

> You can run multiple BSP nodes by changing the `dev-account` flag to `bob` or `charlie`, changing the `download-path` flag to a different directory and change the `port` flag to a different port.

### Run User node

```bash
RUST_LOG=info cargo run -- --run-as user --port 44913 --upload-path "./files-to-upload"
```

This will wait for any file requests from any nodes (this will be improved to wait for specific nodes returned by the runtime) and send the file data.

Output (truncated):

```bash
2023-12-12T20:54:10.444220Z  INFO libp2p_swarm: local_peer_id=12D3KooWDV5MttiC2UGq1tGqsjC51ze89HtNv5xLJGi9XKChwFkq
2023-12-12T20:54:10.444519Z  INFO storagehub_client::p2p::service: Node starting up with peerId PeerId("12D3KooWDV5MttiC2UGq1tGqsjC51ze89HtNv5xLJGi9XKChwFkq")
2023-12-12T20:54:10.444785Z  INFO storagehub_client::p2p::swarm: [SwarmEvent::NewListenAddr] - listen address: /ip4/127.0.0.1/tcp/44913/p2p/12D3KooWDV5MttiC2UGq1tGqsjC51ze89HtNv5xLJGi9XKChwFkq
2023-12-12T20:54:10.444846Z  INFO storagehub_client::p2p::swarm: [SwarmEvent::NewListenAddr] - listen address: /ip4/172.28.164.193/tcp/44913/p2p/12D3KooWDV5MttiC2UGq1tGqsjC51ze89HtNv5xLJGi9XKChwFkq
```

> Any extrinsics executed requiring a Multiaddress will need to be supplied with the `127.0.0.1` address as the nodes are running locally on your machine.
> In the Docker compose environment, the public IP address of the node will need to be used.
