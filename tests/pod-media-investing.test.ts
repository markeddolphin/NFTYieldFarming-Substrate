import BN from 'bn.js';
import { patract, network, artifacts } from 'redspot';

const { getContractFactory, getRandomSigner } = patract;

const { getSigners, api } = network;

const uriAlice =
  'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice';
const uriBob =
  'bottom drive obey lake curtain smoke basket hold race lonely fit walk//Bob';

describe('pod-media-investing', () => {
  after(() => {
    return api.disconnect();
  });

  async function setup() {
    const one = new BN(10).pow(new BN(api.registry.chainDecimals[0]));
    const Alice = (await getSigners())[0];

    // deploy erc20 contract
    const erc20Factory = await getContractFactory('erc20', Alice);
    const daiContract = await erc20Factory.deployed(
      'new_optional',
      '1000000000000000000000000000000000',
      'DAI',
      'DAI',
      12
    );

    const erc1620Factory = await getContractFactory('erc1620', Alice);
    const erc1620Contract = await erc1620Factory.deployed('new');

    const erc721Factory = await getContractFactory('erc721', Alice);
    const erc721Contract = await erc721Factory.deployed('new');

    const mediaFactory = await getContractFactory('media', Alice);
    const mediaContract = await mediaFactory.deployed(
      'new',
      erc1620Contract.address,
      erc721Contract.address
    );

    const podMediaInvesting = await getContractFactory(
      'pod_media_investing',
      Alice
    );

    return {
      Alice,
      erc20Factory,
      erc1620Contract,
      erc721Contract,
      mediaContract,
      daiContract,
      one,
      podMediaInvesting
    };
  }

  it('Can create a pod', async () => {
    let { podMediaInvesting, daiContract, mediaContract, Alice } =
      await setup();

    let inititiate_investing_pod_request = {
      pod_token_symbol: 'PODDAI',
      pod_token_name: 'PODDAI',
      funding_token: daiContract.address,
      funding_token_price: 280000,
      funding_target: 28000000000,
      amm: 'Quadratic',
      spread: 1,
      max_price: 300000,
      max_supply: 150000000,
      funding_date: 1629237600000,
      erc20_code_hash: daiContract.abi.project.source.wasmHash,
      media_contract: mediaContract.address,
      medias: [
        {
          name: 'test media',
          type: 'Blog',
          view_conditions: {
            viewing_type: 'Fixed',
            viewing_token: daiContract.address,
            price: 50,
            sharing_percent: 10,
            is_streaming_live: false,
            streaming_proportions: [],
            token_reward: [],
            token_entry: [],
            duration: 10000000000
          },
          nft_conditions: {
            funding_token: daiContract.address,
            price: 5000
          },
          royalty: 50,
          collabs: [[Alice.address, 1_000_000_000]]
        }
      ]
    };
    await podMediaInvesting.deploy('new', inititiate_investing_pod_request);
  });
});
