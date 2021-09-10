import {patract, network} from 'redspot';
const {getContractFactory} = patract;
const {getSigners} = network;
import {Summary} from "./summary";

const contractName = 'exchange';

async function deploy(signer, summary: Summary, params?) {
    const exchangeFactory = await getContractFactory(contractName, signer);
    let contract = await exchangeFactory.deployed('new', { ...params });

    summary.contracts.push({Contract: contractName, AccountId: contract.address.toHuman()})
    summary.deployed[contractName] = contract;
}

export default  {
    contractName,
    deploy
}