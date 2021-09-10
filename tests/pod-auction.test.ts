import { expect } from 'chai';
import { patract, network } from 'redspot';

const { getContractFactory } = patract;
const { getSigners, api } = network;

describe('POD AUCTION', () => {
  after(() => {
    return api.disconnect();
  });

  const ONE_SECOND = 1000;

  async function setup() {
    await api.isReady;
    const signers = await getSigners();
    const Alice = signers[0];
    const Bob = signers[1];

    // deploy erc20 contract
    const erc20contractFactory = await getContractFactory('erc20', Alice);
    const erc20contract = await erc20contractFactory.deployed('new', '10000');

    // deploy erc721 contract
    const erc721contractFactory = await getContractFactory('erc721', Alice);
    const erc721contract = await erc721contractFactory.deployed('new');

    // deploy pod auction
    const podAuctionContractFactory = await getContractFactory(
      'pod_auction',
      Alice
    );
    const podAuctionContract = await podAuctionContractFactory.deployed('new');

    return {
      podAuctionContract,
      erc20contract,
      erc721contract,
      Alice,
      Bob
    };
  }

  async function isEuropa(): Promise<Boolean> {
    let methods = (await api.rpc.rpc.methods()).methods;
    return methods.includes('europa_backwardToHeight');
  }

  it('Create, Bid and withdraw auction works', async () => {
    const { podAuctionContract, erc20contract, erc721contract, Alice, Bob } =
      await setup();

    // Bob creates erc 721
    await expect(erc721contract.connect(Bob).tx.mint(Bob.address)).to.emit(
      erc721contract,
      'Transfer'
    );
    const ownerOf = await erc721contract.query.ownerOf(1);
    expect(ownerOf.output).to.equal(Bob.address);

    // Bob approve pod auction to spend erc721
    await expect(
      erc721contract.connect(Bob).tx.approve(podAuctionContract.address, 1)
    ).to.emit(erc721contract, 'Approval');

    // Alice (contract owner) add Bob & herself to approval list
    await podAuctionContract.tx.approveUser(Bob.address);
    await podAuctionContract.tx.approveUser(Alice.address);

    // Bob creates an auction for its token
    const time = await podAuctionContract.query.getBlockTimeStamp();
    let now = Number(time.output);

    let startTimeSeconds = 12;
    if (await isEuropa()) {
      startTimeSeconds = 3;
    }

    await expect(
      podAuctionContract.connect(Bob).tx.createAuction({
        media_address: erc721contract.address,
        media_token_id: 1,
        token_address: erc20contract.address,
        owner: Bob.address,
        bid_increment: 10,
        start_time: now + ONE_SECOND * startTimeSeconds,
        end_time: now + ONE_SECOND * 100,
        reserve_price: 1,
        ipfs_hash: 'ipfs hash'
      })
    ).to.emit(podAuctionContract, 'AuctionCreated');

    // Alice allow pod auction contract to spend erc20 on her behalf
    await expect(
      erc20contract.tx.approve(podAuctionContract.address, 10000)
    ).to.emit(erc20contract, 'Approval');

    //Check Alice balance before bid
    const balanceOfAlice = await erc20contract.query.balanceOf(Alice.address);
    expect(balanceOfAlice.output).to.equal(10000);

    // Alice place a bid on the auction
    await expect(
      podAuctionContract.tx.placeBid({
        token_address: erc20contract.address,
        owner: Bob.address,
        amount: 1000
      })
    ).to.emit(podAuctionContract, 'BidPlaced');

    // Check auction gathered (bid amount)
    const auctionsQuery = await podAuctionContract.query.getActiveAuctions();
    // @ts-ignore
    expect(auctionsQuery.output.length).to.equal(1);
    // @ts-ignore
    expect(auctionsQuery.output[0].gathered).to.equal(1000);

    // Check Alice balance after bid
    const balanceOfAliceAfterBid = await erc20contract.query.balanceOf(
      Alice.address
    );
    expect(balanceOfAliceAfterBid.output).to.equal(9000);

    // Check Bob's erc20 balance before with withraw
    const balanceOfBob = await erc20contract.query.balanceOf(Bob.address);
    expect(balanceOfBob.output).to.equal(0);

    //check that auction is not withdrawn
    const auction1 = await podAuctionContract.query.getAuctionByPair(
      erc20contract.address,
      Bob.address
    );
    // @ts-ignore
    expect(auction1.output.unwrap().withdrawn).to.equal(false);

    // withdraw auction
    const withdrawAuction = podAuctionContract.connect(Bob).tx.withdrawAuction({
      token_address: erc20contract.address,
      owner: Bob.address
    });
    await expect(withdrawAuction).to.emit(
      podAuctionContract,
      'AuctionWithdrawn'
    );

    // check Bob's erc20 balance
    const balanceOfBobAfter = await erc20contract.query.balanceOf(Bob.address);
    expect(balanceOfBobAfter.output).to.equal(1000);

    // check that Alice owns the nft
    const ownerOfNft = await erc721contract.query.ownerOf(1);
    expect(ownerOfNft.output).to.equal(Alice.address);

    //check that auction is withdrawn
    const auctionsQuery2 = await podAuctionContract.query.getActiveAuctions();
    // @ts-ignore
    await expect(auctionsQuery2.output.length).to.equal(0);
  }).timeout(120000);

  it('Reset & Cancel auction works', async () => {
    const { podAuctionContract, erc20contract, erc721contract, Alice, Bob } =
      await setup();

    // Bob creates erc 721
    await expect(erc721contract.connect(Bob).tx.mint(Bob.address)).to.emit(
      erc721contract,
      'Transfer'
    );
    const ownerOf = await erc721contract.query.ownerOf(1);
    expect(ownerOf.output).to.equal(Bob.address);

    // Bob approve pod auction to spend erc721
    await expect(
      erc721contract.connect(Bob).tx.approve(podAuctionContract.address, 1)
    ).to.emit(erc721contract, 'Approval');

    // Alice (contract owner) add Bob & herself to approval list
    await podAuctionContract.tx.approveUser(Bob.address);
    await podAuctionContract.tx.approveUser(Alice.address);

    // Bob creates an auction for its token
    const time = await podAuctionContract.query.getBlockTimeStamp();
    let now = Number(time.output);

    let startTimeSeconds = 12;
    if (await isEuropa()) {
      startTimeSeconds = 3;
    }

    await expect(
      podAuctionContract.connect(Bob).tx.createAuction({
        media_address: erc721contract.address,
        media_token_id: 1,
        token_address: erc20contract.address,
        owner: Bob.address,
        bid_increment: 10,
        start_time: now + ONE_SECOND * startTimeSeconds,
        end_time: now + ONE_SECOND * 100,
        reserve_price: 1,
        ipfs_hash: 'ipfs hash'
      })
    ).to.emit(podAuctionContract, 'AuctionCreated');

    // Alice allow pod auction contract to spend erc20 on her behalf
    await expect(
      erc20contract.tx.approve(podAuctionContract.address, 10000)
    ).to.emit(erc20contract, 'Approval');

    // Alice place a bid on the auction
    await expect(
      podAuctionContract.tx.placeBid({
        token_address: erc20contract.address,
        owner: Bob.address,
        amount: 1000
      })
    ).to.emit(podAuctionContract, 'BidPlaced');

    // Check auction gathered (bid amount)
    const auctionsQuery = await podAuctionContract.query.getActiveAuctions();
    // @ts-ignore
    await expect(auctionsQuery.output.length).to.equal(1);
    // @ts-ignore
    await expect(auctionsQuery.output[0].gathered).to.equal(1000);

    // Reset auction
    await expect(
      podAuctionContract.connect(Bob).tx.resetAuction({
        media_address: erc721contract.address,
        media_token_id: 1,
        token_address: erc20contract.address,
        owner: Bob.address,
        bid_increment: 100,
        end_time: now + ONE_SECOND * 100,
        reserve_price: 10,
        ipfs_hash: 'ipfs hash'
      })
    ).to.emit(podAuctionContract, 'AuctionReset');

    // Check that nft still hold the nft
    const ownerOfNft = await erc721contract.query.ownerOf(1);
    expect(ownerOfNft.output).to.equal(podAuctionContract.address);

    // check that Alice get back the bid funds
    const balanceOfAlice = await erc20contract.query.balanceOf(Alice.address);
    expect(balanceOfAlice.output).to.equal(10000);

    //check that auction is reset with new params
    const auction = await podAuctionContract.query.getAuctionByPair(
      erc20contract.address,
      Bob.address
    );
    // @ts-ignore
    expect(auction.output.unwrap().bid_increment).to.equal(100);
    // @ts-ignore
    expect(auction.output.unwrap().reserve_price).to.equal(10);

    // Replace a bid
    await expect(
      podAuctionContract.tx.placeBid({
        token_address: erc20contract.address,
        owner: Bob.address,
        amount: 1000
      })
    ).to.emit(podAuctionContract, 'BidPlaced');

    // check Alice balance after bid
    const balanceOfAlice2 = await erc20contract.query.balanceOf(Alice.address);
    expect(balanceOfAlice2.output).to.equal(9000);

    // Cancel Auction
    await expect(
      podAuctionContract.connect(Bob).tx.cancelAuction({
        token_address: erc20contract.address,
        owner: Bob.address
      })
    ).to.emit(podAuctionContract, 'AuctionCanceled');

    // Check that nft is reverted back to Bob
    const ownerOfNft2 = await erc721contract.query.ownerOf(1);
    expect(ownerOfNft2.output).to.equal(Bob.address);

    // check that Alice get back the bid funds
    const balanceOfAlice3 = await erc20contract.query.balanceOf(Alice.address);
    expect(balanceOfAlice3.output).to.equal(10000);
  }).timeout(120000);
});
