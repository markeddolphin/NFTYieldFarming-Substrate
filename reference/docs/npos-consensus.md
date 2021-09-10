# Switching from PoA to NPoS

By Default the Substrate Node template is configured with PoA consensus algorithm.
For PoA consensus the pallet used alongside with GrandPa is Aura.
In Order to swith from PoA to NPoS we need to use the BABE pallet.
You can find more informations about consensus in substrate [here](https://substrate.dev/docs/en/knowledgebase/advanced/consensus).

## Modifiaction Steps
We will need to modify:
- files in runtime folder
- files in node folder

You can refer to the Susbtrate [repo](https://github.com/paritytech/substrate/tree/master/bin/node/runtime) on how it is setup for the runtime.

For the Node folder also we will need to remove references to Aura pallet and replace them with Babe to modify the.
- [chain_spec.rs](https://github.com/Privi-Protocol/Privi-Substrate/blob/develop/node/src/chain_spec.rs) file
- [runtime.rs](https://github.com/Privi-Protocol/Privi-Substrate/blob/develop/node/src/service.rs) file
- [cargo.toml](https://github.com/Privi-Protocol/Privi-Substrate/blob/develop/node/Cargo.toml) file

This section will be updated in the future.

## Generating chain specification file

After we are done on configuring the NPoS, we need to generate a custom chain spec json file.
This file will allow other nodes that want to be validators and boot from the PRIVI Network or Testnet.

We supposed we have the project already built with (`cargo build --release`) first generate the customSpec.json file by running:

`./target/release/privi build-spec --disable-default-bootnode --chain local > customSpec.json`

We see at the root of the project the customSpec.json file.
We will modify it by adding keys for Grandpa Babe, pallet_session, pallet_staking,...

## Generating Keys

In general for generating a keys with subkey we proceed as follows :

Run the command `subkey generate --scheme` and  we will see the following result
`scheme` can be `sr25519` or `ed25519`

```yaml
subkey generate
Secret phrase `<word1 word2 word3 ... wordX>` is account:
  Secret seed:      0x_seed
  Public key (hex): 0x_pubic_key_Account_ID
  Account ID:       0x_Pubic_key_Account_ID
  SS58 Address:     validator_SS58_address
```

We will need keys for grandpa, babe for finalizing blocks in the consensus
We also might need keys for im_online and authority_discovery but for now that's not the case.

You can refer to this [tutorial](https://substrate.dev/docs/en/tutorials/start-a-private-network/keygen) on how to use Subkey.

We will need to generate keys for

- pallet_balance (key for each validator)
- pallet_session
- pallet_sudo

we can refer to the section below to have an idea on tthe numbers of keys to generate and the scheme used.


### Generating Keys for Pallet balances

```json
"palletBalances": {
  "balances": [
    [
      "validator_SS58_address_1",    <-- Generated Address
      1000000000000000000 <-- Validator Balance we setup
    ],
    [
      "validator_SS58_address_2",    <-- Generated Address
      1000000000000000000 <-- Validator Balance we setup
    ],
    ....,

    [
      "sudo_SS58_address",    <-- Generated Address
      1000000000000000000 <-- Sudo Balance we setup
    ],
  ]
},
```
### Generating Keys for Pallet Session

```json
[
    "%validator_SS58_address%",
    "%validator_SS58_address%",
    {
        "babe": "%sr25519_babe_SS58_address%",
        "im_online": "%sr25519_im_online_SS58_address%",
        "authority_discovery":"%sr25519_authority_discovery_SS58_address%",
        "grandpa": "%ed25519_grandpa_SS58_address%",
    }
]
```

### Generating Keys for Pallet Sudo
```json
...
 "palletSudo": {
        "key": "%sudo_SS58_address%"
      },
...
```

## Generating the raw chain specification file 
`./target/release/privi build-spec --chain=customSpec.json --raw --disable-default-bootnode > customSpecRaw.json`
We will share this file with other fellow validator nodes.


## Running the first validator node

We run the command by

```
./target/release/privi \
  --base-path data/node01 \
  --chain ./customSpecRaw.json \
  --port 30333 \
  --ws-port 9944 \
  --rpc-port 9933 \
  --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' \
  --validator \
  --rpc-methods Unsafe \
  --name PriviNode1 \
  --unsafe-ws-external \
  --rpc-cors all
  ```
After the node starts we that the local Node Identity that we will keep for other nodes to boot from (with bootnodes option)


## Inserting Keys in First Node

for the node we do wia ssh a curl call to insert babe and grandpa public keys inside the keystore of the running node within the ssh session.

- For grandpa we do

`
curl http://127.0.0.1:9933 -H "Content-Type:application/json;charset=utf-8" -d '{"jsonrpc":"2.0","id":11,"method":"author_insertKey","params":["gran","eager over crucial test hybrid ask simple clever trial rotate job galaxy","0x_long_long_long_public_key"]}'
`

- For Babe we do

`
curl http://127.0.0.1:9933 -H "Content-Type:application/json;charset=utf-8" -d '{"jsonrpc":"2.0","id":12,"method":"author_insertKey","params":["babe","urge tackle speed dash vocal retire quote carbon before faith what mansion","0x74efe919a714630a5e4394efa3b8eaed507312cf941d590491d6bf5d5f6a0e0f"]}'
`

## Running Other validator node

We also run this node by using the customSpecRaw file

```
./target/release/privi \
  --base-path data/node02 \
  --chain ./customSpecRaw.json \
  --port 30337 \
  --ws-port 9947 \
  --rpc-port 9937 \
  --telemetry-url 'wss://telemetry.polkadot.io/submit/ 0' \
  --validator \
  --rpc-methods Unsafe \
  --name PriviNode2 \
  --bootnodes /ip4/134.122.29.246/tcp/30333/p2p/first_node_local_identity_key
```


## Inserting Keys in other nodes

For each Node we repeat the same steps as we did in inserting keys for the first node(Each node has it's own keys) with the curl call within the ssh session of the node.

## Stop Nodes and Restart

At this stage we are almost done we need lastly to stop all nodes and restart them to see NPoS working and block produced and finalized.

