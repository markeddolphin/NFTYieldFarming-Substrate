import { expect } from 'chai';
import { network, patract } from 'redspot';

const { getContractFactory, getRandomSigner } = patract;
const { api, getSigners } = network;

describe('exchange', () => {
  after(() => {
    return api.disconnect();
  });

  async function setup() {
    await api.isReady;
    const Alice = (await getSigners())[0];

    // deploy erc20 contracts
    const erc20Factory = await getContractFactory('erc20', Alice);
    const priviContract = await erc20Factory.deployed('new', '10000');
    const usdtContract = await erc20Factory.deployed('new', '10000');

    // deploy exchange
    const exchangeContractFactory = await getContractFactory('exchange', Alice);
    const exchangeContract = await exchangeContractFactory.deployed('new');

    return {
      exchangeContract,
      exchangeContractFactory,
      erc20Factory,
      priviContract,
      usdtContract,
      Alice
    };
  }

  it('Performs all functions', async () => {
    const { exchangeContract, priviContract, usdtContract, Alice } =
      await setup();

    const account1 = await getRandomSigner(Alice, '10 UNIT');
    const account2 = await getRandomSigner(Alice, '10 UNIT');

    // set up privi funds
    await expect(
      priviContract.connect(account1).tx.approve(exchangeContract.address, 2000)
    ).to.emit(priviContract, 'Approval');
    await expect(
      priviContract.connect(account2).tx.approve(exchangeContract.address, 2000)
    ).to.emit(priviContract, 'Approval');
    await priviContract.tx.transfer(account1.address, 1000);
    await priviContract.tx.transfer(account2.address, 1000);

    // set up usdt funds
    await expect(
      usdtContract.connect(account1).tx.approve(exchangeContract.address, 2000)
    ).to.emit(usdtContract, 'Approval');
    await expect(
      usdtContract.connect(account2).tx.approve(exchangeContract.address, 2000)
    ).to.emit(usdtContract, 'Approval');
    await usdtContract.tx.transfer(account1.address, 1000);
    await usdtContract.tx.transfer(account2.address, 1000);

    const priviToken = { account_id: priviContract.address, standard: 'Erc20' };
    const usdtToken = { account_id: usdtContract.address, standard: 'Erc20' };

    // create an exchange
    let events = (
      await exchangeContract.connect(account1).tx.createExchange({
        exchange_token: priviToken,
        initial_amount: '10',
        offer_token: usdtToken,
        price: `2`
      })
    ).events;

    // make sure exchange exists
    let exchangeId = events[1].args[0].exchange_id;
    const firstOfferId = events[1].args[0].offer_id;
    let result = await exchangeContract.query.getExchangeById(exchangeId);
    expect(result.output.unwrap().price).to.equal(2);

    // make sure offer exists
    result = await exchangeContract.query.getExchangeOffers(exchangeId);
    expect(result.output.unwrap()[0].amount).to.equal(10);

    // make sure tokens were transferred from account1 to exchange
    expect(
      (await priviContract.query.balanceOf(account1.address)).output
    ).to.equal(990);
    expect(
      (await priviContract.query.balanceOf(exchangeContract.address)).output
    ).to.equal(10);

    // account2 places an offer to buy
    events = (
      await exchangeContract.connect(account2).tx.placeBuyingOffer({
        exchange_id: exchangeId,
        address: account2.address,
        offer_token: usdtToken,
        amount: 5,
        price: 3
      })
    ).events;
    let buyingOfferId = events[1].args[0].offer_id;

    expect(
      (await usdtContract.query.balanceOf(account2.address)).output
    ).to.equal(985);
    expect(
      (await usdtContract.query.balanceOf(exchangeContract.address)).output
    ).to.equal(15);

    // account2 places an offer to sell
    events = (
      await exchangeContract.connect(account2).tx.placeSellingOffer({
        exchange_id: exchangeId,
        address: account2.address,
        offer_token: usdtToken,
        amount: 3,
        price: 2
      })
    ).events;
    let sellingOfferId = events[1].args[0].offer_id;

    expect(
      (await priviContract.query.balanceOf(account2.address)).output
    ).to.equal(997);
    expect(
      (await priviContract.query.balanceOf(exchangeContract.address)).output
    ).to.equal(13);

    // check if offers exist
    result = await exchangeContract.query.getExchangeOffers(exchangeId);
    const offers = result.output.unwrap();
    expect(offers[1].id).to.equal(buyingOfferId);
    expect(offers[2].id).to.equal(sellingOfferId);

    // account2 cancels their buying offer
    await expect(
      exchangeContract.connect(account2).tx.cancelBuyingOffer({
        exchange_id: exchangeId,
        offer_id: buyingOfferId
      })
    ).to.emit(exchangeContract, 'CanceledOffer');

    expect(
      (await usdtContract.query.balanceOf(account2.address)).output
    ).to.equal(1000);
    expect(
      (await usdtContract.query.balanceOf(exchangeContract.address)).output
    ).to.equal(0);

    // account2 cancels their selling offer
    await expect(
      exchangeContract.connect(account2).tx.cancelSellingOffer({
        exchange_id: exchangeId,
        offer_id: sellingOfferId
      })
    ).to.emit(exchangeContract, 'CanceledOffer');

    expect(
      (await priviContract.query.balanceOf(account2.address)).output
    ).to.equal(1000);
    expect(
      (await priviContract.query.balanceOf(exchangeContract.address)).output
    ).to.equal(10);

    // make sure the offers no longer exist
    result = await exchangeContract.query.getExchangeOffers(exchangeId);
    expect(result.output.unwrap().length).to.equal(1);

    // account2 buys from account1's original offer
    events = (
      await exchangeContract.connect(account2).tx.buyFromOffer({
        exchange_id: exchangeId,
        offer_id: firstOfferId,
        address: account2.address,
        amount: 1
      })
    ).events;
    expect(events.length).to.equal(2);

    expect(
      (await usdtContract.query.balanceOf(account2.address)).output
    ).to.equal(998);
    expect(
      (await priviContract.query.balanceOf(exchangeContract.address)).output
    ).to.equal(9);
    expect(
      (await priviContract.query.balanceOf(account2.address)).output
    ).to.equal(1001);

    // account2 places another buying offer
    events = (
      await exchangeContract.connect(account2).tx.placeBuyingOffer({
        exchange_id: exchangeId,
        address: account2.address,
        offer_token: usdtToken,
        amount: 5,
        price: 3
      })
    ).events;
    buyingOfferId = events[1].args[0].offer_id;

    expect(
      (await usdtContract.query.balanceOf(account2.address)).output
    ).to.equal(983);
    expect(
      (await usdtContract.query.balanceOf(exchangeContract.address)).output
    ).to.equal(15);

    // account1 sells to the offer
    events = (
      await exchangeContract.connect(account1).tx.sellFromOffer({
        exchange_id: exchangeId,
        offer_id: buyingOfferId,
        address: account1.address,
        amount: 3
      })
    ).events;
    expect(events.length).to.equal(2);

    expect(
      (await priviContract.query.balanceOf(exchangeContract.address)).output
    ).to.equal(6);
    expect(
      (await priviContract.query.balanceOf(account1.address)).output
    ).to.equal(990);
    expect(
      (await usdtContract.query.balanceOf(exchangeContract.address)).output
    ).to.equal(6);
    expect(
      (await usdtContract.query.balanceOf(account1.address)).output
    ).to.equal(1011);
  });
});
