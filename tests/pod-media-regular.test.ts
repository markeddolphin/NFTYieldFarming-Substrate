import BN from 'bn.js';
import { expect } from 'chai';
import { patract, network } from 'redspot';

const { getContractFactory } = patract;

const { getSigners, api } = network;

describe('pod-media-regular', () => {
  after(() => {
    return api.disconnect();
  });

  async function setup() {
    await api.isReady;
    const Alice = (await getSigners())[0];

    // deploy erc20 contract
    const erc20Factory = await getContractFactory('erc20', Alice);
    const daiContract = await erc20Factory.deployed(
      'new',
      '1000000000000000000000000000000000'
      /* 'new_optional',
      1000000000000000000000000000000000,
      'DAI',
      'DAI',
      12*/
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

    const podMediaRegularFactory = await getContractFactory(
      'pod_media_regular',
      Alice
    );

    return {
      Alice,
      erc20Factory,
      erc1620Contract,
      erc721Contract,
      mediaContract,
      daiContract,
      podMediaRegularFactory
    };
  }

  it('Can create a pod', async () => {
    let { podMediaRegularFactory, daiContract, mediaContract, Alice } =
      await setup();
    let pod = await podMediaRegularFactory.deploy('new', {
      erc20_code_hash: daiContract.abi.project.source.wasmHash,
      endowment: 1000000000,
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
    });

    let result = await pod.query.creator();
    expect(result.output).to.equal(Alice.address);
  });
});
