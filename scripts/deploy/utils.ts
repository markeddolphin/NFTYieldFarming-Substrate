import ContractFactory from '@redspot/patract/contractFactory';
import { Summary } from './summary';
import Contract from '@redspot/patract/contract';

export default async function getOrDeploy(factory: ContractFactory, summary: Summary, name: string, ...config: any[]): Promise<Contract> {
  if (summary.deployed[name] == undefined || summary.deployed[name] == null) {
    // @ts-ignore
    let contract = await factory.deployed(...config);
    summary.contracts.push({Contract: name, AccountId: contract.address.toHuman()})
    summary.deployed[name] = contract
    return contract
  }
  return summary.deployed[name]
}