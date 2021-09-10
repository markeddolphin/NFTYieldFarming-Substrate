# Privi <> BTC Bridge Design

## Possibilities

There are two ways in general when building bridges:

- Centralized way (using a trusted authority) 
- Decentralized and trustless way


In a decentralized fashion, we would need protocols like XCLAIM, since our goal is to bridge to Bitcoin based network and this type of network doesn't support smart contracts and it's not based on Substrate.

This protocol  in particular, requires any swappable asset to be backed by a `collateral` of higher value than the swappable assets, which adds additional overhead.

The following links might me a good starting point

[ChainX](https://github.com/chainx-org/ChainX)

[Interlay PolkaBTC](https://interlay.gitlab.io/polkabtc-spec/)

[XCLAIM](https://eprint.iacr.org/2018/643.pdf)

[references](https://wiki.polkadot.network/docs/en/learn-bridges)