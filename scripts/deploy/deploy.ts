import { network } from 'redspot';
import { writeSummary} from './summary';

const { createSigner, keyring, api } = network;

import tokenAccounts  from './deployTokenAccounts';
import exchange from './deployExchange';
import htlcAtomicSwap from './deployHtlcAtomicSwap';
import stableCoin from './deployStableCoin';
import erc721 from './deployErc721';
import erc1620 from './deployErc1620';
import media from './deployMedia';

import { Summary } from './summary';
const uri =
    'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice';

async function run() {
  await api.isReady;

  const signer = createSigner(keyring.createFromUri(uri));
  const balance = await api.query.system.account(signer.address);

  console.log("deploying using account: ", signer.address)
  console.log("deploying with balance: ", balance.toHuman())

  let args = {
    gasLimit: "200000000000",
    value: "10000000000000000",
    salt: "12312",
    endowment: '0',
  }

  let summary: Summary = {
    contracts: [],
    deployed: {}
  }

  await tokenAccounts.deploy(signer, summary, args);
  await exchange.deploy(signer, summary, args);
  await htlcAtomicSwap.deploy(signer, summary, args);
  await stableCoin.deploy(signer, summary, args);
  await erc721.deploy(signer, summary, args);
  await erc1620.deploy(signer, summary, args);
  await media.deploy(signer, summary, args);

  writeSummary("deployment-summary.md", summary)
  await api.disconnect();
}

run().catch((err) => {
  console.log(err);
});
