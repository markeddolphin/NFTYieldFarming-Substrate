import { network, patract } from 'redspot';
import { Summary } from './summary';
import getOrDeploy from './utils';

const {getContractFactory} = patract;

const contractName = "erc721"


async function deploy(signer, summary: Summary, params?) {
  const erc721ContractFactory = await getContractFactory(contractName, signer)
  return await getOrDeploy(erc721ContractFactory, summary, contractName, "new", params);
}

export default {
  contractName,
  deploy
}

