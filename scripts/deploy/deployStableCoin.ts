import { patract, network, artifacts } from 'redspot';
import {Summary} from "./summary";
import getOrDeploy from './utils';
const {getContractFactory} = patract;

const contractName = "stable_coin";

// Deploys the stable-coin contract. For stable-coin to work, the following must happen:
// - the collateral token (Privi) must be created.
// - the stablecoin token (pUSD) must be created.
// - the stable-coin contract must be created, with the addresses of the collateral token and stablecoin token.
// - the stable-coin contract must have mint and burn roles for both tokens.
async function deploy(signer, summary: Summary, params?) {
  let privi = {name: "Privi",  symbol: "Privi", supply: '1000'};
  let pusd = {name: "PUSD", symbol: "PUSD",  supply: '1000'};

  const erc20Factory = await getContractFactory('erc20', signer);
  let priviContract = await getOrDeploy(erc20Factory, summary, "erc20(Privi)", 'new_optional', privi.supply, privi.name, privi.symbol, undefined, params)
  let pusdContract = await getOrDeploy(erc20Factory, summary, "erc20(PUSD)", 'new_optional', pusd.supply, pusd.name, pusd.symbol, undefined, params)

  const stableCoinFactory = await getContractFactory(contractName, signer);
  const stableCoinContract = await stableCoinFactory.deployed(
    'new_raw',
    {
      decimalCount: 12,
      accountId: pusdContract.address,
      ticker: pusd.symbol,
      },
    {
      decimalCount: 12,
      accountId: priviContract.address,
      ticker: privi.symbol,
    },
    ...params
  );

  await pusdContract.tx.addMinter(stableCoinContract.address)
  await pusdContract.tx.addBurner(stableCoinContract.address)
  await priviContract.tx.addMinter(stableCoinContract.address)
  await priviContract.tx.addBurner(stableCoinContract.address)
  summary.contracts.push({Contract: contractName, AccountId: stableCoinContract.address.toHuman()})
}

export default {
  contractName,
  deploy
}