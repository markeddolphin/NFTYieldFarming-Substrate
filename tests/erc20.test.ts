import { expect } from 'chai';
import { network, patract } from 'redspot';

const { getContractFactory, getRandomSigner } = patract;
const { api, getSigners } = network;

describe('ERC20', () => {
  after(() => {
    return api.disconnect();
  });

  async function setup() {
    await api.isReady;
    const Alice = (await getSigners())[0];

    const contractFactory = await getContractFactory('erc20', Alice);
    const contract = await contractFactory.deploy('new', '1000');
    const receiver = await getRandomSigner(Alice, '10 UNIT');

    return { contractFactory, contract, receiver, Alice };
  }

  it('Assigns initial balance', async () => {
    const { contract, Alice } = await setup();
    const result = await contract.query.balanceOf(Alice.address);
    expect(result.output).to.equal(1000);
  });

  it('Transfer adds amount to destination account', async () => {
    const { contract, receiver } = await setup();

    await expect(() =>
      contract.tx.transfer(receiver.address, 7)
    ).to.changeTokenBalance(contract, receiver, 7);

    await expect(() =>
      contract.tx.transfer(receiver.address, 7)
    ).to.changeTokenBalances(contract, [contract.signer, receiver], [-7, 7]);
  });

  it('Transfer emits event', async () => {
    const { contract, Alice, receiver } = await setup();

    await expect(contract.tx.transfer(receiver.address, 7))
      .to.emit(contract, 'Transfer')
      .withArgs(Alice.address, receiver.address, 7);
  });

  it('Can not transfer above the amount', async () => {
    const { contract, receiver } = await setup();

    await expect(contract.tx.transfer(receiver.address, 1007)).to.not.emit(
      contract,
      'Transfer'
    );
  });

  it('Can not transfer from empty account', async () => {
    const { contract, Alice } = await setup();

    const emptyAccount = await getRandomSigner(Alice, '10 UNIT');

    await expect(
      contract.connect(emptyAccount).tx.transfer(Alice.address, 7)
    ).to.not.emit(contract, 'Transfer');
  });

  it('Mint increases supply', async () => {
    const { contract, receiver } = await setup();
    await expect(contract.tx.mint(receiver.address, 200), 'mint').to.emit(
      contract,
      'Transfer'
    );

    let balance = await contract.query.balanceOf(receiver.address);
    expect(balance.output).to.eq(200);
  });

  it('Burn decreases supply', async () => {
    const { contract, receiver } = await setup();
    await expect(contract.tx.mint(receiver.address, 200), 'mint').to.emit(
      contract,
      'Transfer'
    );

    let balance = await contract.query.balanceOf(receiver.address);
    expect(balance.output).to.eq(200);

    let totalSupply = await contract.totalSupply();
    await expect(totalSupply.output).to.equal(1200);

    await contract.tx.addBurner(receiver.address);
    await expect(contract.connect(receiver).tx.burn(100)).to.emit(
      contract,
      'Transfer'
    );

    totalSupply = await contract.totalSupply();
    await expect(totalSupply.output).to.equal(1100);

    balance = await contract.query.balanceOf(receiver.address);
    expect(balance.output).to.eq(100);
  });
});
