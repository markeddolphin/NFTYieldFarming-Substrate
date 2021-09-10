import { expect } from 'chai';
import { network, patract } from 'redspot';

const { getContractFactory, getRandomSigner } = patract;
const { api, getSigners } = network;

describe('erc1620', () => {
  after(() => {
    return api.disconnect();
  });

  async function setup() {
    await api.isReady;
    const Alice = (await getSigners())[0];

    // deploy erc20 contracts
    const erc20Factory = await getContractFactory('erc20', Alice);
    const erc20Contract = await erc20Factory.deployed('new', '10000');

    // deploy erc1620
    const erc1620Factory = await getContractFactory('erc1620', Alice);
    const erc1620Contract = await erc1620Factory.deployed('new');

    const bob = await getRandomSigner(Alice, '10 UNIT');
    const charlie = await getRandomSigner(Alice, '10 UNIT');

    // set up bob funds
    await expect(
      erc20Contract.connect(bob).tx.approve(erc1620Contract.address, 2000)
    ).to.emit(erc20Contract, 'Approval');
    await erc20Contract.tx.transfer(bob.address, 100);

    return {
      erc1620Contract,
      erc20Factory,
      erc20Contract,
      Alice,
      bob,
      charlie
    };
  }

  it('Can create and cancel stream', async () => {
    const { erc20Contract, erc1620Contract, bob, charlie } = await setup();

    let block = await api.rpc.chain.getBlock();
    // console.log(`block ${block.block.extrinsics[0].args[0]}`);
    let lastBlockTime = block.block.extrinsics[0].args[0].toNumber();
    let blockNumber = (await api.rpc.chain.getHeader()).number.toNumber();
    // console.log(`blockNumber: ${blockNumber}`);

    // why do I have to add 3 seconds?
    let now = lastBlockTime + 6000;

    let events = (
      await erc1620Contract
        .connect(bob)
        .tx.createStream(
          charlie.address,
          10,
          erc20Contract.address,
          now,
          now + 1000
        )
    ).events;

    // make sure the funds were transferred
    let balance = await erc20Contract.query.balanceOf(erc1620Contract.address);
    expect(balance.output).to.equal(10);
    balance = await erc20Contract.query.balanceOf(bob.address);
    expect(balance.output).to.equal(90);

    // check the generated stream id
    const streamId = events[1].args[0];
    expect(streamId).to.equal(1);

    // validate the stream
    let result = await erc1620Contract.query.getStream(streamId);
    const stream = result.output.unwrap();
    expect(stream.deposit).to.equal(10);

    // const delay = ms => new Promise(res => setTimeout(res, ms));
    // await delay(5000);

    // blockNumber = (await api.rpc.chain.getHeader()).number.toNumber();
    // console.log(`blockNumber: ${blockNumber}`);

    // await api.rpc.europa.forwardToHeight(blockNumber + 3);

    // check balances
    // result = await erc1620Contract.query.balanceOf(streamId, bob.address);
    // expect(result.output).to.equal(0);
    // result = await erc1620Contract.query.balanceOf(streamId, charlie.address);
    // expect(result.output).to.equal(10);

    // cancel the stream
    await erc1620Contract.tx.cancelStream(streamId);

    // the stream should no longer exist
    result = await erc1620Contract.query.getStream(streamId);
    expect(result.output.isNone).to.equal(true);

    // funds should be returned
    balance = await erc20Contract.query.balanceOf(erc1620Contract.address);
    expect(balance.output).to.equal(0);
    // balance = await erc20Contract.query.balanceOf(bob.address);
    // expect(balance.output).to.equal(90);
    // balance = await erc20Contract.query.balanceOf(charlie.address);
    // expect(balance.output).to.equal(10);
  });
});
