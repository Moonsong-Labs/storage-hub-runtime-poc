# PoC: Main Storage Provider

> This is a proof of concept and is by no means a finished product.

## Features

- [ ] `subxt` (light client)
  - [x] Connect to substrate node
  - [x] Subscribe to events
  - [ ] Send extrinsics
- [ ] `libp2p` (peer-to-peer networking)
  - [x] MSP node sends file request
  - [ ] User validates MSP node is registered on chain
  - [x] Establish connection between User and MSP node
  - [x] Send file data
  - [x] Receive file data
  - [ ] MSP node validates data against `content_hash`

## Demo

### Upload file from User client to MSP

1. In one terminal clone and run the substrate node template

    ```bash
    git clone https://github.com/Moonsong-Labs/storage-hub-runtime-poc/tree/poc/michael

    git checkout poc/michael

    cargo build --release

    ./target/release/node-template --dev
    ```

2. In two other separate terminals, run the User client and MSP:

    **MSP node**

    ```bash
    # MSP node
    cargo run -- --secret-key-seed 1 --run-as msp-provider --chain local --port 35435
    ```

    This will connect to the substrate node template via `subxt` (which utilizes `smoldot` in the background) and will subscribe to events being triggered by the substrate node.

    It will specifically listen to the `RequestStore` event which contains all the information required for the MSP node to send a file request to the user's peer address which should contain the file.

    Output (truncated):

    ```bash
    INFO libp2p_swarm: local_peer_id=12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X
    INFO main_sp: Multiaddr listening on /ip4/127.0.0.1/tcp/35435/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X
    INFO main_sp: Multiaddr listening on /ip4/172.28.164.193/tcp/35435/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X
    INFO main_sp::client: Connected to Development network using ws://127.0.0.1:9944 * Substrate node Substrate Node vRuntimeVersion { spec_version: 100, transaction_version: 1 }
    INFO main_sp::runtimes::local: Subscribe 'RequestStore' on-chain finalized event
    ```

    **User client**

    ```bash
    cargo run -- --secret-key-seed 2 --run-as user --port 44913
    ```

    This will wait for any file requests from any nodes (this will be improved to wait for specific nodes returned by the runtime) and send the file data.

    Output (truncated):

    ```bash
    INFO libp2p_swarm: local_peer_id=12D3KooWH3uVF6wv47WnArKHk5p6cvgCJEb74UTmxztmQDc298L3
    INFO main_sp: Multiaddr listening on /ip4/127.0.0.1/tcp/44913/p2p/12D3KooWH3uVF6wv47WnArKHk5p6cvgCJEb74UTmxztmQDc298L3
    INFO main_sp: Multiaddr listening on /ip4/172.28.164.193/tcp/44913/p2p/12D3KooWH3uVF6wv47WnArKHk5p6cvgCJEb74UTmxztmQDc298L3
    ```

3. Go to the [nuclear dashboard](https://cloudflare-ipfs.com/ipns/dotapps.io/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/extrinsics) and run the `request_store` extrinsic in the template pallet.

    You can use the [substrate utilities](https://www.shawntabrizi.com/substrate-js-utilities/) (String to Hex) to easily convert the values to the appropriate format.

4. The file content will be printed in the MSP terminal.
