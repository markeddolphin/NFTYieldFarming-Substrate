import {patract, network} from 'redspot';
const {getContractFactory} = patract;
const {getSigners} = network;
import {Summary} from "./summary";

const contractName = "htlc_atomic_swap";

async function deploy(signer, summary: Summary, params?) {
    const htlcFactory = await getContractFactory(contractName, signer);
    let contract = await htlcFactory.deployed('new',  ...params );

    summary.contracts.push({Contract: contractName, AccountId: contract.address.toHuman()})
    summary.deployed[contractName] = contract;
}

export default {
    contractName,
    deploy
}