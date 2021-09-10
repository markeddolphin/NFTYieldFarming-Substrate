import { network, patract } from 'redspot';
import { Summary } from './summary';
import getOrDeploy from './utils';

const {getContractFactory} = patract;

const contractName = "erc1620"

async function deploy(signer, summary: Summary, params?) {
  const erc1620ContractFactory = await getContractFactory(contractName, signer)
  return await getOrDeploy(erc1620ContractFactory, summary, contractName, "new", params);
}

export default {
  contractName,
  deploy
}