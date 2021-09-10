# ChainBridge

## What is ChainBridge?

ChainBridge is an extensible cross-chain communication protocol. It currently supports bridging between EVM and Substrate based chains.

A bridge contract (or pallet in Substrate) on each chain forms either side of a bridge. Handler contracts allow for customizable behavior upon receiving transactions to and from the bridge. For example locking up an asset on one side and minting a new one on the other. Its highly customizable - you can deploy a handler contract to perform any action you like.

In its current state ChainBridge operates under a trusted federation model. Deposit events on one chain are detected by a trusted set of off-chain relayers who await finality, submit events to the other chain and vote on submissions to reach acceptance triggering the appropriate handler.

For more details, please check [ChainBridge-Doc](https://chainbridge.chainsafe.io/).

## Relevant Reopos in ChainBridge

| Repo | Description |
| ------ | ------ |
| [ChainBridge](https://github.com/ChainSafe/ChainBridge) | This is the core bridging software that Relayers run between chains. |
| [chainbridge-solidity](https://github.com/ChainSafe/chainbridge-solidity) | The Solidity contracts required for chainbridge. Includes deployment and interaction CLI. |
| [chainbridge-substrate](https://github.com/ChainSafe/chainbridge-substrate) | A substrate pallet that can be integrated into a chain, as well as an example pallet to demonstrate chain integration. |
| [chainbridge-utils](https://github.com/ChainSafe/chainbridge-utils) | A collection of packages used by the core bridging software. |
| [chainbridge-deploy](https://github.com/ChainSafe/chainbridge-deploy) | Some tooling to help with deployments. |

## How do we use ChainBridge in PRIVI?

We use ChainBridge for bridging between `Privi-Substrate` and EVM chains. In this documentation, it describes how to setup ChainBridge for `Ropsten Testnet` vs `Privi-Substrate`.

## Why do we have a seperate documentation here?

Actually, ChainBridge has a great documentation here. [ChainBridge-Doc](https://chainbridge.chainsafe.io/).

The following reasons are why we have a seperate document:

- First point: `Privi-Substrate` is based on substrate `v3`. However, the current ChainBridge's repositories are not yet updated fully to be compatible with substrate v3. (Of course, we all believe that ChainBridge will update it soon. But we can't wait until that. :) ). So we need some modification to ChainBridge's codebase to make it compatible with substrate v3. This document includes that.

- Second point: ChainBridge Doc describes the setup between the substrate vs local evm. However, this document setups the ChainBridge between `Ropsten Testnet` vs `Privi-Substrate`.

✨ Sounds exciting? ✨ Let's go!

---
## Steps one by one

#### 1. Download `Privi-Substrate` repo and run the substrate node.

```sh
git clone git@github.com:Privi-Protocol/Privi-Substrate.git
cd Privi-Substrate
cargo build --release
./target/release/privi --dev --tmp
```

#### 2. Polkadot JS Portal (https://polkadot.js.org/)

- Open the Polkadot JS Portal and connect to your local node by clicking in the top-left corner and using `ws://localhost:9944`.
- You will need to setup the type definitions for the chain by selecting Settings -> Developer
- Here is the updated Type definitions to be compatible with substrate v3.
```
{
    "chainbridge::ChainId": "u8",
    "ChainId": "u8",
    "ResourceId": "[u8; 32]",
    "DepositNonce": "u64",
    "ProposalVotes": {
      "votes_for": "Vec<MultiAddress>",
      "votes_against": "Vec<MultiAddress>",
      "status": "enum"
    },
    "Erc721Token": {
      "id": "TokenId",
      "metadata": "Vec<u8>"
    },
    "TokenId": "U256",
    "Address": "MultiAddress",
    "LookupSource": "MultiAddress"
  }
```

#### 3. Deploy ChainBridge solidity smart contracts to Ropsten

Here is the ChainBridge's solidity smart contract repo. [chainbridge-solidity](https://github.com/ChainSafe/chainbridge-solidity)

ChainBridge also provides a CLI deployment tool [cb-sol-cli](https://github.com/ChainSafe/chainbridge-deploy/tree/master/cb-sol-cli) which helps us to deploy chainbridge's smart contracts easily.

```sh
git clone git@github.com:ChainSafe/chainbridge-deploy.git
cd chainbridge-deploy/cb-sol-cli
```

Now, let's change the deployerAddress.

Go to `cb-sol-cli/constants.js` and make the following changes.
```
module.exports.deployerAddress = "Your Address";
module.exports.deployerPrivKey = "Your Private Key";
```

Also, let's change relayerAddresses.

Go to `cb-sol-cli/constants.js` and make the following changes.
```
module.exports.relayerAddresses = [
    "xxx",
    "yyy",
    "zzz",
]

module.exports.adminAddresses = [
    "aaa",
    "bbb",
]

module.exports.relayerPrivKeys = [
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "ccccccccccccccccccccccccccc",
]
```

At this point, we need some modification to cb-sol-cli codebase for Ropsten deploy. (Of course, there could be another way without code modification but I was more comfortable with updating the code.)

We can connect to Ropsten in two ways. Either works.

1. Using Ropsten Default Provider

- Go to `cb-sol-cli/utils.js` and make the following changes.
```
    args.provider = ethers.getDefaultProvider('ropsten');
    // if (!parent.networkId) {
    //     args.provider = new ethers.providers.JsonRpcProvider(args.url);
    // } else {
    //     args.provider = new ethers.providers.JsonRpcProvider(args.url, {
    //         name: "custom",
    //         chainId: Number(parent.networkId)
    //     });
    // }
```

2. Using Infura

- Sign up to https://infura.io and create a new project. Go to the `Settings` and select the `ENDPOINT` as `Ropsten`. And copy the wss connection url. It should be similar to `wss://ropsten.infura.io/ws/v3/xxx`

- Go to `cb-sol-cli/index.js` and replace "http://localhost:8545" with `wss://ropsten.infura.io/ws/v3/xxx`. Set `networkId` as 3. (Ropsten)

----

Let's build the cb-sol-cli using the following command
```sh
make install
```

Now, we are ready to deploy ChainBridge's smart contracts using `cb-sol-cli` to Ropsten.

```sh
cb-sol-cli deploy --all --relayerThreshold 1
```

After running, the expected output looks like this:
```sh
================================================================
Url:        xxx
Deployer:   xxx
Gas Limit:   8000000
Gas Price:   20000000
Deploy Cost: 0.0

Options
=======
Chain Id:    0
Threshold:   1
Relayers:    0xff93B45308FD417dF303D6515aB04D9e89a750Ca,0x8e0a907331554AF72563Bd8D43051C2E64Be5d35,0x24962717f8fA5BA3b931bACaF9ac03924EB475a0,0x148FfB2074A9e59eD58142822b3eB3fcBffb0cd7,0x4CEEf6139f00F9F4535Ad19640Ff7A0137708485
Bridge Fee:  0
Expiry:      100

Contract Addresses
================================================================
Bridge:             0x25607015933Ac23B109AdD909A19f973D3a20320
----------------------------------------------------------------
Erc20 Handler:      0xf2d39D9604a2685CF584b272554ee551f5666D86
----------------------------------------------------------------
Erc721 Handler:     0xACEcfa44631d4433Aec7E38704F4B0Eb558e2584
----------------------------------------------------------------
Generic Handler:    0x50B7835bA0e88b5945E4A0e403d11Bebd16FdcBd
----------------------------------------------------------------
Erc20:              0xF29940Efab7922BdF07fCD304049aBb8Eed27a0D
----------------------------------------------------------------
Erc721:             0x74baFC5cFC9ed5B4d0C8758aE57b72788FaE6F50
----------------------------------------------------------------
Centrifuge Asset:   Not Deployed
----------------------------------------------------------------
WETC:               Not Deployed
================================================================
```

> Note: `As we deploy the smart contracts on Ropsten, the contract addresses are not the same with the above. So please consider this and replace with your addresses from the next section.`

#### 4. Register Resources (on Ropsten)

```sh
MY_WALLET=0xeC44513a4204b031d5A6D562E09cf0a229e35ae5
ADDR_BRIDGE=0x25607015933Ac23B109AdD909A19f973D3a20320
ADDR_ERC20_HANDLER=0xf2d39D9604a2685CF584b272554ee551f5666D86
ADDR_ERC721_HANDLER=0xACEcfa44631d4433Aec7E38704F4B0Eb558e2584
ADDR_GENERIC_HANDLER=0x50B7835bA0e88b5945E4A0e403d11Bebd16FdcBd
ADDR_ERC20=0xF29940Efab7922BdF07fCD304049aBb8Eed27a0D
ADDR_ERC721=0x74baFC5cFC9ed5B4d0C8758aE57b72788FaE6F50
```

```sh
cb-sol-cli admin add-relayer --relayer "0xeC44513a4204b031d5A6D562E09cf0a229e35ae5" --bridge "$ADDR_BRIDGE"
```

```sh
# Register fungible resource ID with erc20 contract
cb-sol-cli bridge register-resource --bridge "$ADDR_BRIDGE" --resourceId "0x000000000000000000000000000000c76ebe4a02bbc34786d860b355f5a5ce00" --targetContract "$ADDR_ERC20" --handler "$ADDR_ERC20_HANDLER"

# Register non-fungible resource ID with erc721 contract
cb-sol-cli bridge register-resource --bridge "$ADDR_BRIDGE" --resourceId "0x000000000000000000000000000000e389d61c11e5fe32ec1735b3cd38c69501" --targetContract "$ADDR_ERC721" --handler "$ADDR_ERC721_HANDLER"

# Register generic resource ID
cb-sol-cli bridge register-generic-resource --bridge "$ADDR_BRIDGE" --resourceId "0x000000000000000000000000000000f44be64d2de895454c3467021928e55e01" --targetContract "0xc279648CE5cAa25B9bA753dAb0Dfef44A069BaF4" --handler "$ADDR_GENERIC_HANDLER" --hash --deposit "" --execute "store(bytes32)"
```

#### 5. Specify Token Semantics (on Ropsten)
To allow for a variety of use cases, the Ethereum contracts support both the transfer and the mint/burn ERC methods.

For simplicity's sake the following examples only make use of the mint/burn method:
```sh
# Register the erc20 contract as mintable/burnable
cb-sol-cli bridge set-burn --bridge "$ADDR_BRIDGE" --handler "$ADDR_ERC20_HANDLER" --tokenContract "$ADDR_ERC20"

# Register the associated handler as a minter
cb-sol-cli erc20 add-minter --erc20Address "$ADDR_ERC20" --minter "$ADDR_ERC20_HANDLER"

# Register the erc721 contract as mintable/burnable
cb-sol-cli bridge set-burn --bridge "$ADDR_BRIDGE" --tokenContract "$ADDR_ERC721" --handler "$ADDR_ERC721_HANDLER"

# Add the handler as a minter
cb-sol-cli erc721 add-minter --erc721Address "$ADDR_ERC721" --minter "$ADDR_ERC721_HANDLER"
```

#### 6. Registering Relayers (on Substrate)
First we need to register the account of the relayer on substrate (cb-sol-cli deploys contracts with the 5 test keys preloaded).

Select the `Sudo` tab in the PolkadotJS UI. Choose the `addRelayer` method of `chainBridge`, and select your relayer account as the relayer. (You can generate one on the polkadot explorer. You can also find how to configure this relayer account on chainbridge config file in the later section)

#### 7. Register Resources (on Substrate)

Select the `Sudo` tab and call `chainBridge.setResourceId` for each of the transfer types you wish to use:

Fungible (Native asset):

Id: `0x000000000000000000000000000000c76ebe4a02bbc34786d860b355f5a5ce00`

Method: `0x4578616d706c652e7472616e73666572` (utf-8 encoding of "Example.transfer")

NonFungible(ERC721):

Id: `0x000000000000000000000000000000e389d61c11e5fe32ec1735b3cd38c69501`

Method: `0x4578616d706c652e6d696e745f657263373231` (utf-8 encoding of "Example.mint_erc721")

Generic (Hash Transfer):

Id: `0x000000000000000000000000000000f44be64d2de895454c3467021928e55e01`

Method: `0x4578616d706c652e72656d61726b` (utf-8 encoding of "Example.remark")

#### 8. Whitelist Chains (on Substrate)
Using the `Sudo` tab, call `chainBridge.whitelistChain`, specifying 0 for out ethereum chain ID.

#### 9. Running A Relayer

Current [ChainBridge](https://github.com/ChainSafe/ChainBridge) is not compatible with substrate v3 so I forked it and updated some dependencies and codebase here.

https://github.com/frankli-dev/ChainBridge

Here are the changes in go.mod file:
  * centrifuge/go-substrate-rpc-client/v3 v3.0.0
  * frankli-dev/chainbridge-substrate-events v0.0.0-20210422013950-38a6764c511a
  * frankli-dev/chainbridge-utils v1.0.9 // indirect

So you can clone this repo and run the relayer for compatibility with substrate v3. (But of course, ChainBridge will update this soon on their repositories. After that, you don't need to use this forked repo.)

To run a relayer, we need to add relayer keys to the Chainbridge's keystore. (chainbridge repo/keys)

- To add substrate address
  Generate one account on the polkadotjs explorer. (Or you can use `subkey` command)
  Then, use `./build/chainbridge accounts` command to import it. Here is the example:
  `./build/chainbridge accounts import --sr25519 --privateKey xxx`
  This command will create a key file `./key` folder.
- To add ethereum address

Here is an example config file for a single relayer using the contracts we've deployed.
Please check the config file carefully and replace with your appropriate values.

```
{
  "chains": [
    {
      "name": "eth",
      "type": "ethereum",
      "id": "0",
      "endpoint": [Infura ws connection url. For example: wss://ropsten.infura.io/ws/v3/xxx],
      "from": [Deployer Address],
      "opts": {
        "bridge": [Bridge Contract Address],
        "erc20Handler": [ERC20 Handler Contract Address],
        "erc721Handler": [ERC721 Handler Contract Address],
        "genericHandler": [Generic Handler Contract Address],
        "gasLimit": "8000000",
        "maxGasPrice": "2000000000"
      }
    },
    {
      "name": "sub",
      "type": "substrate",
      "id": "1",
      "endpoint": "ws://localhost:9944",
      "from": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
      "opts": {
        "useExtendedCall": "true"
      }
    }
  ]
}
```

Run `make install`in ChanBridge directory to build chainbridge and put it in GOBIN path,

You can then start a relayer as a binary using the default "Alice" key.

```sh
chainbridge --config config.json --latest
```

#### 10. Fungible Transfers

##### Substrate Native Token ⇒ ERC 20¶
In the substrate UI select the `Extrinsics` tab, and call `example.transferNative` with these parameters:

* Amount: `1000000`
* Recipient: `0xeC44513a4204b031d5A6D562E09cf0a229e35ae5`
* Dest Id: 0

You can query the recipients balance on ethereum with this:

```sh
cb-sol-cli erc20 balance --address "$MY_WALLET" --erc20Address "$ADDR_ERC20"
```

##### ERC20 ⇒ Substrate Native Token
```sh
cb-sol-cli erc20 mint --erc20Address "$ADDR_ERC20" --amount 1000
cb-sol-cli erc20 approve --amount 1000 --recipient "$ADDR_ERC20_HANDLER" --erc20Address "$ADDR_ERC20"
cb-sol-cli erc20 deposit --amount 1 --dest 1 --recipient "0xc65111c63f474d8ce06721646eae6ca7b4e58824880a66bfeb0d6bb588105f3d" --resourceId "0x000000000000000000000000000000c76ebe4a02bbc34786d860b355f5a5ce00" --bridge "$ADDR_BRIDGE"
```

##### Substrate NFT ⇒ ERC721
First, you'll need to mint a token. Select the Sudo tab and call `erc721.mint` with parameters such as these:

* Owner: `Alice`
* TokenId: `1`
* Metadata: `""`

Now the owner of the token can initiate a transfer by calling `example.transferErc721`:

* Recipient: `0xeC44513a4204b031d5A6D562E09cf0a229e35ae5`
* TokenId: `1`
* DestId: `0`

You can query ownership of tokens on ethereum with this:
```sh
cb-sol-cli erc721 owner --id 0x1 --erc721Address "$ADDR_ERC721"
```

##### ERC721 ⇒ Substrate NFT
If necessary, you can mint an erc721 token like this:

```sh
cb-sol-cli erc721 mint --id 0x99 --erc721Address "$ADDR_ERC721" --metadata "My NFT1 from rinkeby"
```

Before initiating the transfer, we must approve the bridge to take ownership of the tokens:

```sh
cb-sol-cli erc721 approve --id 0x99 --erc721Address "$ADDR_ERC721" --recipient "$ADDR_ERC721_HANDLER"
```

Now we can initiate the transfer:

```sh
cb-sol-cli erc721 deposit --id 0x99 --dest 1 --resourceId "0x000000000000000000000000000000e389d61c11e5fe32ec1735b3cd38c69501" --recipient "0x3a96583f4bf563b811626df52c76c92226055a97d980ff995fe07439b24e4857" --bridge "$ADDR_BRIDGE"
```

##### Generic Data Substrate ⇒ Eth¶

For this example we will transfer a 32 byte hash to a registry on ethereum. Using the Extrinsics tab, call `example.transferHash`:

* Hash: `0x699c776c7e6ce8e6d96d979b60e41135a13a2303ae1610c8d546f31f0c6dc730`
* Dest ID: `0`

You can verify the transfer with this command:
.requiredOption('--hash <value>', 'A hash to lookup', ethers.utils.hexZeroPad("0x", 32))
    .option('--address <value>', 'Centrifuge asset store contract address', constants.CENTRIFUGE_ASSET_STORE_ADDRESS)
```sh
cb-sol-cli cent getHash --hash 0x699c776c7e6ce8e6d96d979b60e41135a13a2303ae1610c8d546f31f0c6dc730 --address 
```