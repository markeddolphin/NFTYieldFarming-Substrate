import { RedspotUserConfig } from 'redspot/types';
import '@redspot/patract';
import '@redspot/chai';
import '@redspot/gas-reporter';
import "@redspot/decimals";
import "@redspot/explorer";

import * as types from './reference/additional-types.json'
const {AccountInfo, ...localTypes} = types;

const CONTRACTS = [ 'contracts/**/*' ];

export default {
  defaultNetwork: 'development',
  contract: {
    ink: {
      toolchain: 'nightly',
      sources: CONTRACTS
    }
  },
  networks: {
    development: {
      endpoint: 'ws://127.0.0.1:9944',
      types: { ...localTypes } ,
      gasLimit: '400000000000',
      explorerUrl:
        'https://polkadot.js.org/apps/#/explorer/query/?rpc=ws://127.0.0.1:9944/'
    },
    testnet: {
      endpoint: 'wss://privisubstrate.com',
      gasLimit: '400000000000',
      accounts: ['//Alice'],
      types: { ...localTypes } ,
    },
    ci: {
      endpoint: 'ws://europa:9944',
      types: { ...localTypes } ,
      gasLimit: '400000000000'
    }
  },
  mocha: {
    timeout: 180000
  }
} as RedspotUserConfig;
