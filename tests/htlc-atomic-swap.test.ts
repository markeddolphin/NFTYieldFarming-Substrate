import { expect } from 'chai';
import { network, patract } from 'redspot';

const { getContractFactory, getRandomSigner } = patract;
const { api, getSigners } = network;

describe('htlc-atomic-swap', () => {
  after(() => {
    return api.disconnect();
  });

  async function setup() {
    await api.isReady;
    const Alice = (await getSigners())[0];

    // deploy erc20 contracts
    const erc20Factory = await getContractFactory('erc20', Alice);
    const usdtContract = await erc20Factory.deploy('new', '10000');

    // deploy exchange
    const htlcFactory = await getContractFactory('htlc_atomic_swap', Alice);
    const htlcContract = await htlcFactory.deploy('new');

    return {
      htlcContract,
      htlcFactory,
      erc20Factory,
      usdtContract,
      Alice
    };
  }

  it('Can claim funds', async () => {
    const { htlcContract, usdtContract, Alice } = await setup();

    const bob = await getRandomSigner(Alice, '10 UNIT');
    const charlie = await getRandomSigner(Alice, '10 UNIT');

    // set up usdt funds
    await expect(
      usdtContract.connect(bob).tx.approve(htlcContract.address, 2000)
    ).to.emit(usdtContract, 'Approval');
    await expect(
      usdtContract.connect(charlie).tx.approve(htlcContract.address, 2000)
    ).to.emit(usdtContract, 'Approval');
    await usdtContract.tx.transfer(bob.address, 100);

    // create an HTLC
    const secretHash =
      '0x4c9bf8fc46df3e252c8eaf0d450d7bf95c56f4d6284a3c89af37154dc2660a39';
    let events = (
      await htlcContract.connect(bob).tx.initialiseHtlc({
        to: charlie.address,
        token: { account_id: usdtContract.address, standard: 'Erc20' },
        amount: 100,
        time_lock: 9623316512871,
        secret_hash: secretHash
      })
    ).events;

    // make sure the funds were transferred
    expect(
      (await usdtContract.query.balanceOf(htlcContract.address)).output
    ).to.equal(100);
    expect((await usdtContract.query.balanceOf(bob.address)).output).to.equal(
      0
    );

    // get the contract hash from second event
    let contractHash = events[1].args[0].contract_hash;

    // verify the HTLC was created
    let result = await htlcContract.query.getHtlcInfo(contractHash);
    let output = result.output.unwrap();
    expect(output.secret_hash).to.equal(secretHash);
    expect(output.amount).to.equal(100);

    // redspot is calling this twice for some reason, so I'm just checking the balance
    // claim the funds
    await htlcContract.connect(charlie).tx.claimFunds({
      contract_hash: contractHash,
      secret:
        '0x7e3231d03bb0bd1cd542c20b1ff232e08d88ffd452c576558c9415414a6127ea'
    });
    expect(
      (await usdtContract.query.balanceOf(htlcContract.address)).output
    ).to.equal(0);
    expect(
      (await usdtContract.query.balanceOf(charlie.address)).output
    ).to.equal(100);
  });

  it('Can refund funds', async () => {
    const { htlcContract, usdtContract, Alice } = await setup();

    // TODO: move common code to a function, but I am bad at typescript :(

    const bob = await getRandomSigner(Alice, '10 UNIT');
    const charlie = await getRandomSigner(Alice, '10 UNIT');

    // set up usdt funds
    await expect(
      usdtContract.connect(bob).tx.approve(htlcContract.address, 2000)
    ).to.emit(usdtContract, 'Approval');
    await expect(
      usdtContract.connect(charlie).tx.approve(htlcContract.address, 2000)
    ).to.emit(usdtContract, 'Approval');
    await usdtContract.tx.transfer(bob.address, 100);

    // create an HTLC
    const secretHash =
      '0x4c9bf8fc46df3e252c8eaf0d450d7bf95c56f4d6284a3c89af37154dc2660a39';
    let events = (
      await htlcContract.connect(bob).tx.initialiseHtlc({
        to: charlie.address,
        token: { account_id: usdtContract.address, standard: 0 },
        amount: 100,
        time_lock: 9623316512871,
        secret_hash: secretHash
      })
    ).events;

    // make sure the funds were transferred
    expect(
      (await usdtContract.query.balanceOf(htlcContract.address)).output
    ).to.equal(100);
    expect((await usdtContract.query.balanceOf(bob.address)).output).to.equal(
      0
    );

    // get the contract hash from second event
    let contractHash = events[1].args[0].contract_hash;

    // verify the HTLC was created
    let result = await htlcContract.query.getHtlcInfo(contractHash);
    let output = result.output.unwrap();
    expect(output.secret_hash).to.equal(secretHash);
    expect(output.amount).to.equal(100);

    // redspot is calling this twice for some reason, so I'm just checking the balance
    // refund the funds
    await htlcContract.connect(bob).tx.refundFunds({
      contract_hash: contractHash,
      secret_hash: secretHash
    });
    expect(
      (await usdtContract.query.balanceOf(htlcContract.address)).output
    ).to.equal(0);
    expect((await usdtContract.query.balanceOf(bob.address)).output).to.equal(
      100
    );
  });
});
