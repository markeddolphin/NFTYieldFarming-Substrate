import {patract, network} from 'redspot';
import {expect} from "chai";
import {Summary} from "./summary";

const {getContractFactory} = patract;
const contractName = 'token_accounts';

async function deploy(signer, summary: Summary, params?) {
    let salt = params.salt;

    // deploy token-accounts
    const tokenAccountsFactory = await getContractFactory(contractName, signer);
    const tokenAccountsContract = await tokenAccountsFactory.deployed('new', params );

    // the erc20 tokens that will be deployed
    const erc20Tokens = [
        {name: "Bitcoin", symbol: "BTC", initialSupply: '1000', decimalCount: 12},
        {name: "Ethereum", symbol: "ETH", initialSupply: '1000', decimalCount: 12},
        {name: "Privi",  symbol: "Privi", initialSupply: '1000', decimalCount: 12},
        {name: "PUSD", symbol: "PUSD",  initialSupply: '1000', decimalCount: 12},
    ];

    // deploy erc20 tokens
    const erc20Factory = await getContractFactory('erc20', signer);
    for (const token of erc20Tokens) {
        salt +=1
        let Parameters = { ...params, salt: salt };
        let name = `erc20(${token.symbol})`;
        let contract = await erc20Factory.deployed('new_optional', token.initialSupply, token.name, token.symbol, token.decimalCount, Parameters );
        await expect(tokenAccountsContract.tx.setToken(token.symbol, contract.address, 0)).to.emit(tokenAccountsContract, 'SetToken').withArgs(token.symbol, contract.address, 0);
        console.log(`deployed ${name}`, contract.address.toHuman())
        summary.contracts.push({Contract: `erc20(${token.symbol})`, AccountId: contract.address.toHuman() })
        summary.deployed[name] = contract;
    }
}

export default  {
    contractName,
    deploy
}