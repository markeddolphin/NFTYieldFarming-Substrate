use ink_env::AccountId;
use ink_prelude::vec::Vec;
#[cfg(feature = "std")]
use ink_storage::traits::StorageLayout;
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};


type Balance = <ink_env::DefaultEnvironment as ink_env::Environment>::Balance;

/// The Auction model
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, StorageLayout))]
pub struct AuctionModel {
    /// address of auction owner
    pub owner: AccountId,
    /// auction start time (in unix millisecond)
    pub start_time: u64,
    /// auction end time (in unix millisecond)
    pub end_time: u64,
    /// minimum amount to increase the bid
    pub bid_increment: Balance,
    /// minimum amount to bid
    pub reserve_price: Balance,
    /// Balance gathered in the auction: highest bid
    pub gathered: Balance,
    /// last bidder address
    pub bidder: AccountId,
    /// address of the ERC721  NFT contract (HLF: MediaSymbol)
    pub media_address: AccountId, // HLF: MediaSymbol
    /// id of the Token of the ERC721
    pub media_token_id: u64,
    /// address of the ERC20 contract, HLF: TokenSymbol
    pub token_address: AccountId,
    /// IPFS hash
    pub ipfs_hash: Vec<u8>,
    /// is the auction already withdrawn
    pub withdrawn: bool,
}

/// The create Auction request
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, StorageLayout))]
pub struct CreateAuctionRequest {
    /// address of the ERC721  NFT contract, HLF: MediaSymbol
    pub media_address: AccountId,
    /// id of the Token of the ERC721
    pub media_token_id: u64,
    /// address of the ERC20 contract, HLF: TokenSymbol
    pub token_address: AccountId,
    /// minimum amount to increase the bid
    pub bid_increment: Balance,
    /// auction start time (in unix millisecond)
    pub start_time: u64,
    /// auction end time (in unix millisecond)
    pub end_time: u64,
    /// minimum amount to bid
    pub reserve_price: Balance,
    /// IPFS hash
    pub ipfs_hash: Vec<u8>,
}

/// The place a bid in auction request
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, StorageLayout))]
pub struct PlaceBidRequest {
    /// address of the ERC20 contract, HLF: TokenSymbol
    pub token_address: AccountId,
    /// address of auction owner
    pub owner: AccountId,
    /// amount to bid
    pub amount: Balance,
}

/// The withdraw auction request
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, StorageLayout))]
pub struct WithdrawAuctionRequest {
    /// address of the ERC20 contract, HLF: TokenSymbol
    pub token_address: AccountId,
    /// address of auction owner
    pub owner: AccountId,
}

/// The cancel auction request
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, StorageLayout))]
pub struct CancelAuctionRequest {
    /// address of the ERC20 contract, HLF: TokenSymbol
    pub token_address: AccountId, // HLF: TokenSymbol
    /// address of auction owner
    pub owner: AccountId,
}

/// The reset Auction request
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, StorageLayout))]
pub struct ResetAuctionRequest {
    /// address of the ERC721  NFT contract, HLF: MediaSymbol
    pub media_address: AccountId, // HLF: MediaSymbol
    /// id of the Token of the ERC721
    pub media_token_id: u64,
    /// address of the ERC20 contract, HLF: TokenSymbol
    pub token_address: AccountId, // HLF: TokenSymbol
    /// address of auction owner
    pub owner: AccountId,
    /// minimum amount to increase the bid
    pub bid_increment: Balance,
    /// auction end time (in unix millisecond)
    pub end_time: u64,
    /// minimum amount to bid
    pub reserve_price: Balance,
    /// IPFS hash
    pub ipfs_hash: Vec<u8>,
}

/// Output of an event
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, StorageLayout))]
pub struct Output {
    /// list of Auctions
    pub auctions: Vec<AuctionModel>,
    /// info about transactions
    pub transactions: Vec<Transfer>,
}

/// Transfer info
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, StorageLayout))]
pub struct Transfer {
    pub r#type: Vec<u8>,
    pub token: Vec<u8>,
    pub from: AccountId,
    pub to: AccountId,
    pub amount: Balance,
}
