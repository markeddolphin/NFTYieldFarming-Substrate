import { expect } from 'chai';
import { patract, network } from 'redspot';

const { getContractFactory, getRandomSigner } = patract;
const { getSigners, api } = network;

describe('claimable media', () => {
  after(() => {
    return api.disconnect();
  });

  async function setup() {
    await api.isReady;
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

    const claimableMediaFactory = await getContractFactory(
      'claimable_media',
      Alice
    );

    let createMedia = async () => {
      return await claimableMediaFactory.deploy('new', {
        name: 'media',
        artists: [Alice.address],
        media: mediaContract.address,
        erc1620: erc1620Contract,
        erc20: daiContract,
        view_conditions: {
          viewing_type: 'Dynamic',
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
        }
      });
    };

    return {
      Alice,
      erc20Factory,
      erc1620Contract,
      erc721Contract,
      mediaContract,
      daiContract,
      claimableMediaFactory,
      createMedia
    };
  }

  it('Can create media', async () => {
    let { createMedia } = await setup();

    let media = await createMedia();
  });

  it('should update artist', async () => {
    let { createMedia, Alice } = await setup();
    const artist = await getRandomSigner(Alice, '1 UNIT');

    let media = await createMedia();

    await media.tx.addArtists([artist.address]);
    let result = await media.query.info();
    // @ts-ignore
    expect(result.output.artists.length).to.equal(2);
  });

  it('can set state', async () => {
    let { createMedia, Alice } = await setup();

    let media = await createMedia();

    await media.tx.setState('Claimed');
    let result = await media.query.info();
    // @ts-ignore
    expect(result.output.state).to.equal('Claimed');
  });

  it('can set propose distributions', async () => {
    let { createMedia, Alice } = await setup();

    let media = await createMedia();

    await expect(
      media.connect(Alice.address).tx.proposeDistribution([[Alice.address]])
    )
      .to.emit(media, 'DistributionProposed')
      .withArgs(Alice.address);
  });
});
