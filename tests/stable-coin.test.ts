import BN from 'bn.js';
import { expect } from 'chai';
import { patract, network } from 'redspot';

const { getContractFactory, getRandomSigner } = patract;

const { api, getSigners } = network;

describe('stable-coin', () => {
  after(() => {
    return api.disconnect();
  });

  async function setup() {
    await api.isReady;
    const Alice = (await getSigners())[0];

    // deploy erc20 contract
    const erc20Factory = await getContractFactory('erc20', Alice);
    const priviContract = await erc20Factory.deployed(
      'new_optional',
      '1000000000000000000000000000000000',
      'PRIVI',
      'PRIVI',
      12
    );
    const pUSDContract = await erc20Factory.deployed(
      'new_optional',
      '1000000000000000000000000000000000',
      'PUSD',
      'PUSD',
      12
    );

    // deploy token-accounts
    const tokenAccountsFactory = await getContractFactory(
      'token_accounts',
      Alice
    );
    const tokenAccountsContract = await tokenAccountsFactory.deployed('new');

    // register privi and pUSD
    await expect(
      tokenAccountsContract.tx.setToken('PRIVI', priviContract.address, 0)
    )
      .to.emit(tokenAccountsContract, 'SetToken')
      .withArgs('PRIVI', priviContract.address, 0);
    await expect(
      tokenAccountsContract.tx.setToken('PUSD', pUSDContract.address, 0)
    )
      .to.emit(tokenAccountsContract, 'SetToken')
      .withArgs('PUSD', pUSDContract.address, 0);

    // deploy stable-coin contract
    const stableCoinFactory = await getContractFactory('stable_coin', Alice);
    const stableCoinContract = await stableCoinFactory.deployed(
      'new',
      'PUSD',
      'PRIVI',
      tokenAccountsContract.address
    );
    const receiver = await getRandomSigner(Alice, '500 UNIT');

    return {
      erc20Factory,
      priviContract,
      pUSDContract,
      tokenAccountsContract,
      tokenAccountsFactory,
      stableCoinFactory,
      stableCoinContract,
      receiver,
      oracles: [
        await getRandomSigner(Alice, '5000 UNIT'),
        await getRandomSigner(Alice, '5000 UNIT'),
        await getRandomSigner(Alice, '5000 UNIT'),
        await getRandomSigner(Alice, '5000 UNIT'),
        await getRandomSigner(Alice, '5000 UNIT')
      ]
    };
  }

  it('Registers new oracles', async () => {
    let { stableCoinContract, oracles } = await setup();

    for (const oracle of oracles) {
      await expect(
        stableCoinContract.registerOracle({
          address: oracle.address,
          name: oracle.address
        })
      ).to.emit(stableCoinContract, 'OracleRegistered');
    }
  });

  it('Fails when reregistering existing oracle', async () => {
    let { stableCoinContract, oracles } = await setup();

    for (const oracle of oracles) {
      await expect(
        stableCoinContract.registerOracle({
          address: oracle.address,
          name: oracle.address
        })
      ).to.emit(stableCoinContract, 'OracleRegistered');

      await expect(
        stableCoinContract.registerOracle({
          address: oracle.address,
          name: oracle.address
        })
      ).to.not.emit(stableCoinContract, 'OracleRegistered');
    }
  });

  it('Can only register oracles by the owner', async () => {
    let { stableCoinContract, oracles } = await setup();

    await expect(
      stableCoinContract.connect(oracles[0]).registerOracle({
        address: oracles[0].address,
        name: oracles[0].address
      })
    ).to.not.emit(stableCoinContract, 'OracleRegistered');
  });

  it('Can change oracle state', async () => {
    let { stableCoinContract, oracles } = await setup();

    await expect(
      stableCoinContract.registerOracle({
        address: oracles[0].address,
        name: oracles[0].address
      })
    ).to.emit(stableCoinContract, 'OracleRegistered');

    await expect(
      stableCoinContract.updateOracleState({
        address: oracles[0].address,
        state: 'Allowed'
      })
    ).to.emit(stableCoinContract, 'OracleStateUpdated');
  });

  it('Can only change oracle state by the owner', async () => {
    let { stableCoinContract, oracles } = await setup();

    await expect(
      stableCoinContract.registerOracle({
        address: oracles[0].address,
        name: oracles[0].address
      })
    ).to.emit(stableCoinContract, 'OracleRegistered');

    await expect(
      stableCoinContract.connect(oracles[0]).updateOracleState({
        address: oracles[0].address,
        state: 'Allowed'
      })
    ).to.not.emit(stableCoinContract, 'OracleStateUpdated');
  });

  it('Oracles can submit prices', async () => {
    let { stableCoinContract, oracles } = await setup();

    for (const oracle of oracles) {
      await expect(
        stableCoinContract.registerOracle({
          address: oracle.address,
          name: oracle.address
        })
      ).to.emit(stableCoinContract, 'OracleRegistered');
    }

    for (const oracle of oracles) {
      await expect(
        stableCoinContract.connect(oracle.address).submitPrice({
          token: 'USD',
          price: 1,
          volume: 1
        })
      ).to.emit(stableCoinContract, 'PriceSubmitted');
    }
  });

  it('Computes the current price correctly', async () => {
    let { stableCoinContract, oracles } = await setup();

    const USD = 10000000000;

    for (const oracle of oracles) {
      await expect(
        stableCoinContract.tx.registerOracle({
          address: oracle.address,
          name: oracle.address
        })
      ).to.emit(stableCoinContract, 'OracleRegistered');
    }

    await expect(
      stableCoinContract.connect(oracles[0].address).tx.submitPrice({
        token: 'USD',
        price: USD,
        volume: 500
      })
    ).to.emit(stableCoinContract, 'PriceSubmitted');

    await expect(
      stableCoinContract.connect(oracles[1].address).tx.submitPrice({
        token: 'USD',
        price: 1.2 * USD,
        volume: 1000
      })
    ).to.emit(stableCoinContract, 'PriceSubmitted');

    let p = await stableCoinContract.query.getPrice('USD');
    expect(p.output).to.eq({ Ok: weightedMean([USD, 1.2 * USD], [500, 1000]) });
  });

  it('Converts from USD to Privi', async () => {
    let { stableCoinContract, oracles, pUSDContract, receiver, priviContract } =
      await setup();

    const USD = 10000000000;

    // Init oracles
    for (const oracle of oracles) {
      await expect(
        stableCoinContract.tx.registerOracle({
          address: oracle.address,
          name: oracle.address
        })
      ).to.emit(stableCoinContract, 'OracleRegistered');
    }

    // Set PUSD price
    await expect(
      stableCoinContract.connect(oracles[0].address).tx.submitPrice({
        token: 'PUSD',
        price: USD,
        volume: 500
      })
    ).to.emit(stableCoinContract, 'PriceSubmitted');

    // Set privi price
    await expect(
      stableCoinContract.connect(oracles[1].address).tx.submitPrice({
        token: 'PRIVI',
        price: 10 * USD,
        volume: 1000
      })
    ).to.emit(stableCoinContract, 'PriceSubmitted');

    // Grant roles to contract
    await expect(pUSDContract.tx.addMinter(stableCoinContract.address)).to.emit(
      pUSDContract,
      'AddedMinter'
    );
    await expect(pUSDContract.tx.addBurner(stableCoinContract.address)).to.emit(
      pUSDContract,
      'AddedBurner'
    );
    await expect(
      priviContract.tx.addMinter(stableCoinContract.address)
    ).to.emit(priviContract, 'AddedMinter');
    await expect(
      priviContract.tx.addBurner(stableCoinContract.address)
    ).to.emit(priviContract, 'AddedBurner');

    // Ensure converter has funds
    await expect(
      pUSDContract.tx.transfer(receiver.address, 1000 * USD)
    ).to.emit(pUSDContract, 'Transfer');

    // Grant allowance to swap contract
    await expect(
      pUSDContract
        .connect(receiver)
        .tx.approve(stableCoinContract.address, 500 * USD)
    ).to.emit(pUSDContract, 'Approval');

    // Actual conversion
    await expect(
      stableCoinContract.connect(receiver).tx.convertToPrivi({
        address: receiver.address,
        amount: USD * 500
      })
    ).to.emit(stableCoinContract, 'Conversion');

    let pUSD = await pUSDContract.query.balanceOf(receiver.address);
    expect(pUSD.output).to.equal(500 * USD);

    let privi = await priviContract.query.balanceOf(receiver.address);
    expect(privi.output).to.equal(50 * USD);
  });
});

function weightedMean(arrValues, arrWeights) {
  var result = arrValues
    .map(function (value, i) {
      var weight = arrWeights[i];
      var sum = value * weight;

      return [sum, weight];
    })
    .reduce(
      function (p, c) {
        return [p[0] + c[0], p[1] + c[1]];
      },
      [0, 0]
    );

  return result[0] / result[1];
}
