#![cfg_attr(not(feature = "std"), no_std)]
use ink_lang as ink;

mod models;

#[ink::contract]
mod auction {
    use contract_utils::{TokenStandard, ZERO_ACCOUNT};
    use ink_prelude::{vec, vec::Vec};
    use ink_storage::collections::HashMap as StorageHashMap;
    use multi_token::MultiToken;
    use scale::{Decode, Encode};

    use crate::models::*;

    #[ink(storage)]
    pub struct Auction {
        /// Mapping from (Token Address, Owner) to Auction
        auctions: StorageHashMap<(AccountId, AccountId), AuctionModel>,
        /// Owner of the contract (Account that instantiated the contract)
        owner: AccountId,
        /// Allowed Accounts
        allowed_accounts: StorageHashMap<AccountId, ()>,
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Cannot create an auction that already exist
        AuctionAlreadyExist,
        /// Only Auction Owner allowed
        OnlyAuctionOwnerAllowed,
        /// Error when calling the contract
        Contract,
        /// Invalid time
        InvalidTime,
        /// Only the contract owner is allowed to call the contract
        /// Contract owner is the deployer of the contract
        OnlyOwnerAllowed,
        /// User is already an authorized user
        UserAlreadyAllowed,
        /// User not found in allowed user list
        UserIsNotAllowed,
        /// Auction not found
        AuctionNotFound,
        /// Unable to bid into withdrawn auction
        AuctionHasBeenWithdrawn,
        /// Transfer Error
        Transfer,
        /// Insufficient bid amount
        InsufficientBidAmount,
        /// Cannot withdraw an empty auction
        AuctionHasNoBid,
    }

    /// Event emitted when an auction is created.
    #[ink(event)]
    pub struct AuctionCreated {
        output: Output,
    }

    /// Event emitted when bid is placed
    #[ink(event)]
    pub struct BidPlaced {
        output: Output,
    }

    /// Event emitted when auction is withdrawn
    #[ink(event)]
    pub struct AuctionWithdrawn {
        output: Output,
    }

    /// Event emitted when auction is canceled and it transfer back the tokens
    #[ink(event)]
    pub struct AuctionCanceledTransfer {
        output: Output,
    }

    /// Event emitted when auction is canceled
    #[ink(event)]
    pub struct AuctionCanceled {
        output: Output,
    }

    /// Event emitted when auction is reseted
    #[ink(event)]
    pub struct AuctionReset {
        output: Output,
    }

    /// The Auction result type.
    pub type Result<T> = core::result::Result<T, Error>;

    /// one day in milliseconds.
    const ONE_DAY: u64 = 864_000_000;

    impl Auction {
        #[ink(constructor)]
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            let caller = Self::env().caller();
            Self { auctions: Default::default(), owner: caller, allowed_accounts: Default::default() }
        }

        /// Returns the actual timestamp
        #[ink(message)]
        pub fn get_block_time_stamp(&self) -> u64 { self.env().block_timestamp() }

        /// Returns the auction of the identifier (token_address, owner)
        /// Params:
        /// *token_address: address of the Token
        /// *owner: address of the owner
        #[ink(message)]
        pub fn get_auction_by_pair(&mut self, token_address: AccountId, owner: AccountId) -> Option<AuctionModel> {
            self.auctions.get(&(token_address, owner)).cloned()
        }

        /// Returns the list of approved users
        #[ink(message)]
        pub fn get_approved_users(&self) -> Vec<AccountId> {
            self.allowed_accounts.keys().cloned().collect::<Vec<AccountId>>()
        }

        /// Returns the list of active auction (not withdrawn)
        #[ink(message)]
        pub fn get_active_auctions(&self) -> Vec<AuctionModel> {
            self.auctions.values().filter(|x| !x.withdrawn).cloned().collect()
        }

        /// Approve user and add to approved users
        /// Params:
        /// *user: Account address of the user
        #[ink(message)]
        pub fn approve_user(&mut self, user: AccountId) -> Result<()> {
            self.ensure_contract_owner(self.env().caller())?;
            if self.allowed_accounts.contains_key(&user) {
                return Err(Error::UserAlreadyAllowed);
            }
            self.allowed_accounts.insert(user, ());
            Ok(())
        }

        /// Remove user from approved users
        /// Params:
        /// *user: Account address of the user
        #[ink(message)]
        pub fn remove_user(&mut self, user: AccountId) -> Result<()> {
            self.ensure_contract_owner(self.env().caller())?;
            if !self.allowed_accounts.contains_key(&user) {
                return Err(Error::UserIsNotAllowed);
            }
            self.allowed_accounts.take(&user);
            Ok(())
        }

        /// Create an auction
        /// Params:
        /// *input: CreateAuctionRequest
        #[ink(message)]
        pub fn create_auction(&mut self, input: CreateAuctionRequest) -> Result<()> {
            let caller = self.env().caller();
            self.ensure_allowed_user(caller)?;

            // Check that auction doesn't exist already
            if self.get_auction_by_pair(input.token_address, caller).is_some() {
                return Err(Error::AuctionAlreadyExist);
            }
            // check time
            let now = self.env().block_timestamp();
            if now > input.start_time {
                return Err(Error::InvalidTime);
            }
            if input.start_time >= input.end_time {
                return Err(Error::InvalidTime);
            }

            let current_account_id = self.env().account_id();
            let mut erc721 = MultiToken { account_id: input.media_address, standard: TokenStandard::Erc721 };
            match erc721.transfer_from(caller, current_account_id, Some(input.media_token_id), None) {
                Err(_) => return Err(Error::Transfer),
                Ok(f) => f,
            };

            let auction = AuctionModel {
                owner: caller,
                start_time: input.start_time,
                end_time: input.end_time,
                bid_increment: input.bid_increment,
                reserve_price: input.reserve_price,
                gathered: 0,
                bidder: ZERO_ACCOUNT,
                media_address: input.media_address,
                media_token_id: input.media_token_id,
                token_address: input.token_address,
                ipfs_hash: input.ipfs_hash.clone(),
                withdrawn: false,
            };
            self.auctions.insert((input.token_address, caller), auction.clone());

            self.env().emit_event(AuctionCreated {
                output: Output {
                    auctions: vec![auction],
                    transactions: vec![Transfer {
                        r#type: "transfer".as_bytes().to_vec(),
                        token: "Erc721".as_bytes().to_vec(),
                        from: caller,
                        to: current_account_id,
                        amount: 1,
                    }],
                },
            });

            Ok(())
        }

        /// Create an auction
        /// Params:
        /// *input: CreateAuctionRequest
        #[ink(message)]
        pub fn place_bid(&mut self, input: PlaceBidRequest) -> Result<()> {
            let caller = self.env().caller();
            self.ensure_allowed_user(caller)?;

            let mut auction =
                self.get_auction_by_pair(input.token_address, input.owner).ok_or(Error::AuctionNotFound)?;
            // check time
            let now = self.env().block_timestamp();
            if now < auction.start_time || now > auction.end_time {
                return Err(Error::InvalidTime);
            }
            if auction.withdrawn {
                return Err(Error::AuctionHasBeenWithdrawn);
            }
            if input.amount <= (auction.gathered + auction.bid_increment) {
                return Err(Error::InsufficientBidAmount);
            }
            if input.amount <= auction.reserve_price {
                return Err(Error::InsufficientBidAmount);
            }

            // Send bid to contract. If success save bid in storage
            let is_first_bid = auction.bidder == ZERO_ACCOUNT;
            let current_account_id = self.env().account_id();
            let mut erc20 = MultiToken { account_id: auction.token_address, standard: TokenStandard::Erc20 };
            match erc20.transfer_from(caller, current_account_id, None, Some(input.amount)) {
                Err(_) => Err(Error::Transfer),
                Ok(_) => {
                    // transfer last amount to preceding bidder
                    if !is_first_bid {
                        match erc20.transfer(auction.bidder, None, Some(auction.gathered)) {
                            Err(_) => return Err(Error::Transfer),
                            Ok(f) => f,
                        };
                    }

                    let last_bidder = auction.bidder;
                    auction.gathered = input.amount;
                    auction.bidder = caller;
                    self.auctions.insert((input.token_address, input.owner), auction.clone());

                    let mut transactions = vec![Transfer {
                        r#type: "transfer".as_bytes().to_vec(),
                        token: "Erc721".as_bytes().to_vec(),
                        from: caller,
                        to: current_account_id,
                        amount: 1,
                    }];
                    if !is_first_bid {
                        transactions.push(Transfer {
                            r#type: "transfer".as_bytes().to_vec(),
                            token: "Erc20".as_bytes().to_vec(),
                            from: current_account_id,
                            to: last_bidder,
                            amount: input.amount,
                        });
                    }

                    self.env().emit_event(BidPlaced { output: Output { auctions: vec![auction], transactions } });

                    Ok(())
                }
            }
        }

        /// Withdraw an auction
        /// Params:
        /// *input: WithdrawAuctionRequest
        #[ink(message)]
        pub fn withdraw_auction(&mut self, input: WithdrawAuctionRequest) -> Result<()> {
            let caller = self.env().caller();
            let mut auction =
                self.get_auction_by_pair(input.token_address, input.owner).ok_or(Error::AuctionNotFound)?;

            self.ensure_auction_owner(auction.owner, caller)?;
            if auction.withdrawn {
                return Err(Error::AuctionHasBeenWithdrawn);
            }
            if auction.bidder == ZERO_ACCOUNT {
                return Err(Error::AuctionHasNoBid);
            }

            // ERC721 transferred to bidder
            let mut erc721 = MultiToken { account_id: auction.media_address, standard: TokenStandard::Erc721 };
            match erc721.transfer(auction.bidder, Some(auction.media_token_id), None) {
                Err(_) => return Err(Error::Transfer),
                Ok(f) => f,
            }

            // Amount of ERC20 is transferred to owner
            let mut erc20 = MultiToken { account_id: auction.token_address, standard: TokenStandard::Erc20 };
            match erc20.transfer(auction.owner, None, Some(auction.gathered)) {
                Err(_) => return Err(Error::Transfer),
                Ok(f) => f,
            }

            auction.withdrawn = true;
            auction.gathered = 0;
            self.auctions.insert((input.token_address, input.owner), auction.clone());

            self.env().emit_event(AuctionWithdrawn {
                output: Output {
                    auctions: vec![auction.clone()],
                    transactions: vec![
                        Transfer {
                            r#type: "transfer".as_bytes().to_vec(),
                            token: "Erc721".as_bytes().to_vec(),
                            from: self.env().account_id(),
                            to: caller,
                            amount: 1,
                        },
                        Transfer {
                            r#type: "transfer".as_bytes().to_vec(),
                            token: "Erc20".as_bytes().to_vec(),
                            from: self.env().account_id(),
                            to: auction.bidder,
                            amount: auction.gathered,
                        },
                    ],
                },
            });

            Ok(())
        }

        /// Cancel an auction
        /// Params:
        /// *input: CancelAuctionRequest
        #[ink(message)]
        pub fn cancel_auction(&mut self, input: CancelAuctionRequest) -> Result<()> {
            let caller = self.env().caller();
            let auction = self.get_auction_by_pair(input.token_address, input.owner).ok_or(Error::AuctionNotFound)?;

            self.ensure_auction_owner(auction.owner, caller)?;

            if self.env().block_timestamp() > (auction.end_time + ONE_DAY) {
                return Err(Error::InvalidTime);
            }
            if auction.withdrawn {
                return Err(Error::AuctionHasBeenWithdrawn);
            }

            //Transfer to last bidder
            let is_first_bid = auction.bidder == ZERO_ACCOUNT;
            if !is_first_bid {
                let mut erc20 = MultiToken { account_id: auction.token_address, standard: TokenStandard::Erc20 };
                match erc20.transfer(auction.bidder, None, Some(auction.gathered)) {
                    Err(_) => return Err(Error::Transfer),
                    Ok(f) => f,
                }
            }

            // Transfer ERC721 back to owner
            let mut erc721 = MultiToken { account_id: auction.media_address, standard: TokenStandard::Erc721 };
            match erc721.transfer(auction.owner, Some(auction.media_token_id), None) {
                Err(_) => return Err(Error::Transfer),
                Ok(f) => f,
            }

            let last_bidder = auction.bidder;
            self.auctions.take(&(auction.token_address, auction.owner));

            let mut transactions = vec![Transfer {
                r#type: "transfer".as_bytes().to_vec(),
                token: "Erc721".as_bytes().to_vec(),
                from: self.env().account_id(),
                to: caller,
                amount: 1,
            }];
            if !is_first_bid {
                transactions.push(Transfer {
                    r#type: "transfer".as_bytes().to_vec(),
                    token: "Erc20".as_bytes().to_vec(),
                    from: self.env().account_id(),
                    to: last_bidder,
                    amount: auction.gathered,
                });
            }

            self.env().emit_event(AuctionCanceled { output: Output { auctions: vec![auction], transactions } });

            Ok(())
        }

        /// Reset an auction
        /// Params:
        /// *input: ResetAuctionRequest
        #[ink(message)]
        pub fn reset_auction(&mut self, input: ResetAuctionRequest) -> Result<()> {
            let caller = self.env().caller();
            let mut auction =
                self.get_auction_by_pair(input.token_address, input.owner).ok_or(Error::AuctionNotFound)?;

            self.ensure_auction_owner(auction.owner, caller)?;

            if self.env().block_timestamp() > (auction.end_time + ONE_DAY) {
                return Err(Error::InvalidTime);
            }
            if auction.withdrawn {
                return Err(Error::AuctionHasBeenWithdrawn);
            }
            let now = self.env().block_timestamp();
            if now > input.end_time {
                return Err(Error::InvalidTime);
            }

            //Transfer to last bidder
            let is_first_bid = auction.bidder == ZERO_ACCOUNT;
            if !is_first_bid {
                let mut erc20 = MultiToken { account_id: auction.token_address, standard: TokenStandard::Erc20 };
                match erc20.transfer(auction.bidder, None, Some(auction.gathered)) {
                    Err(_) => return Err(Error::Transfer),
                    Ok(f) => f,
                }
            }

            let last_bidder = auction.bidder;
            let amount_transferred_to_bidder = auction.gathered;
            auction.owner = input.owner;
            auction.media_address = input.media_address;
            auction.media_token_id = input.media_token_id;
            auction.bid_increment = input.bid_increment;
            auction.reserve_price = input.reserve_price;
            auction.ipfs_hash = input.ipfs_hash.clone();
            auction.end_time = input.end_time;
            auction.start_time = now;
            auction.gathered = 0;
            auction.bidder = ZERO_ACCOUNT;

            self.auctions.insert((input.token_address, input.owner), auction.clone());

            let mut transactions: Vec<Transfer> = vec![];
            if !is_first_bid {
                transactions.push(Transfer {
                    r#type: "transfer".as_bytes().to_vec(),
                    token: "Erc20".as_bytes().to_vec(),
                    from: self.env().account_id(),
                    to: last_bidder,
                    amount: amount_transferred_to_bidder,
                });
            }

            self.env().emit_event(AuctionReset { output: Output { auctions: vec![auction], transactions } });

            Ok(())
        }

        /// Ensure that caller is the owner of the auction
        /// Params:
        /// *owner: AccountId of the auction owner
        /// *caller: AccountId of the caller
        #[ink(message)]
        pub fn ensure_auction_owner(&mut self, owner: AccountId, caller: AccountId) -> Result<()> {
            if owner != caller {
                return Err(Error::OnlyAuctionOwnerAllowed);
            }
            Ok(())
        }

        /// Ensure that caller is an allowed account
        /// Params:
        /// *caller: AccountId of the caller
        #[ink(message)]
        pub fn ensure_allowed_user(&mut self, caller: AccountId) -> Result<()> {
            if !self.allowed_accounts.contains_key(&caller) {
                return Err(Error::UserIsNotAllowed);
            }
            Ok(())
        }

        /// Ensure that caller is the owner of the contract
        /// contract owner is set when contract is deployed
        /// Params:
        /// *caller: AccountId of the caller
        #[ink(message)]
        pub fn ensure_contract_owner(&mut self, caller: AccountId) -> Result<()> {
            if caller != self.owner {
                return Err(Error::OnlyOwnerAllowed);
            }
            Ok(())
        }
    }
}
