import { network, patract } from 'redspot';
import { Summary } from './summary';
import getOrDeploy from './utils';

const {getContractFactory} = patract;
const {getSigners} = network;

import erc721 from './deployErc721'
import erc1620 from './deployErc1620';

const contractName = "media";

async function deploy(signer, summary: Summary, params?) {

  const erc1620ContractFactory = await getContractFactory(erc1620.contractName, signer)
  const erc1620Contract = await getOrDeploy(erc1620ContractFactory, summary, erc1620.contractName, "new", params)

  const erc721ContractFactory = await getContractFactory(erc721.contractName, signer)
  const erc721Contract = await getOrDeploy(erc721ContractFactory, summary, erc721.contractName, "new", params)

  const mediaFactory = await getContractFactory(contractName, signer);
  const contract = await mediaFactory.deployed(
    'new',
    erc1620Contract.address,
    erc721Contract.address,
    params
  );
  summary.contracts.push({Contract: contractName, AccountId: contract.address.toHuman()})
  summary.deployed[contractName] = contract
}

export default {
  contractName,
  deploy
}