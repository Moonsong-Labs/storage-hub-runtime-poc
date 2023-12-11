# PoC: Main Storage Provider

> This is a proof of concept and is by no means a finished product.

## Features

- [x] `subxt` (light client)
  - [x] Connect to substrate node
  - [x] Subscribe to events
  - [x] Send extrinsics
  - [x] BSP node dispatches `bsp_volunteer` extrinsic prior to sending file request
- [ ] `libp2p` (peer-to-peer networking)
  - [x] BSP node sends file request
  - [ ] User validates BSP node is registered on chain
  - [x] Establish connection between User and BSP node
  - [x] Send file data
  - [x] Receive file data
  - [ ] BSP/BSP node validates data against `content_hash`

## How to run without Docker compose

### Build and run the substrate node

In one terminal run the storagehub runtime:

```bash
# project-root/storagehub-runtim
cargo build --release

./target/release/node-template --dev
```

### Run BSP node

```bash
cargo run -- --secret-key-seed 1 --run-as bsp-provider --chain local --port 35435 --download-path "/tmp/downloaded-files"
```

This will connect to the substrate node template via `subxt` (which utilizes `smoldot` in the background) and will subscribe to events being triggered by the substrate node.

It will specifically listen to the `RequestStore` event which contains all the information required for the BSP node to send a file request to the user's peer address which should contain the file.

Output (truncated):

```bash
INFO libp2p_swarm: local_peer_id=12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X
INFO main_sp: Multiaddr listening on /ip4/127.0.0.1/tcp/35435/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X
INFO main_sp: Multiaddr listening on /ip4/172.28.164.193/tcp/35435/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X
INFO main_sp::client: Connected to Development network using ws://127.0.0.1:9944 * Substrate node Substrate Node vRuntimeVersion { spec_version: 100, transaction_version: 1 }
INFO main_sp::runtimes::local: Subscribe 'RequestStore' on-chain finalized event
```

### Run User node

```bash
cargo run -- --secret-key-seed 2 --run-as user --port 44913 --upload-path "./files-to-upload"
```

This will wait for any file requests from any nodes (this will be improved to wait for specific nodes returned by the runtime) and send the file data.

Output (truncated):

```bash
INFO libp2p_swarm: local_peer_id=12D3KooWH3uVF6wv47WnArKHk5p6cvgCJEb74UTmxztmQDc298L3
INFO main_sp: Multiaddr listening on /ip4/127.0.0.1/tcp/44913/p2p/12D3KooWH3uVF6wv47WnArKHk5p6cvgCJEb74UTmxztmQDc298L3
INFO main_sp: Multiaddr listening on /ip4/172.28.164.193/tcp/44913/p2p/12D3KooWH3uVF6wv47WnArKHk5p6cvgCJEb74UTmxztmQDc298L3
```
