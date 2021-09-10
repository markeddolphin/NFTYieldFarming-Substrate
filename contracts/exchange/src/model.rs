pub use input::*;
pub use output::*;
pub use storage::*;

use contract_utils::env_exports::*;
use ink_storage::traits::{PackedLayout, SpreadLayout};
use multi_token::{UniqueMultiToken, UniqueMultiTokenInfo};
use scale::{Decode, Encode};

/// A unique identifier for an Exchange
pub type ExchangeId = Hash;

/// A unique identifier for an Offer
pub type OfferId = Hash;

pub mod storage {
    use super::*;

    /// An exchange between two tokens
    #[derive(Debug, Encode, Decode, SpreadLayout, PackedLayout, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout))]
    pub struct Exchange {
        /// Unique ID that represents the exchange
        pub id: ExchangeId,
        /// Creator of the exchange
        pub creator: AccountId,
        /// Token that is going to be traded through this order book model
        pub exchange_token: UniqueMultiToken,
        /// InitialAmount of exchangeToken to sell
        pub initial_amount: Balance,
        /// Price per each exchange token
        pub price: Balance,
    }

    /// An offer for the exchange. Can be a buy or sell offer.
    #[derive(Debug, Encode, Decode, SpreadLayout, PackedLayout, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout))]
    pub struct Offer {
        /// Id identifies the offer
        pub id: OfferId,
        /// ExchangeId identifies the initial offer
        pub exchange_id: ExchangeId,
        /// Type can be Buy or Sell
        pub offer_type: OfferType,
        /// CreatorAddress of the creator of the offer
        pub creator: AccountId,
        /// Price in which the order was placed
        pub price: Balance,
        /// Amount of the token that the owner offers
        pub amount: Balance,
        /// token of the offer
        pub token: UniqueMultiToken,
    }

    /// A type of offer (buy or sell)
    #[derive(Debug, Encode, Decode, SpreadLayout, PackedLayout, Copy, Clone, Eq, PartialEq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout))]
    pub enum OfferType {
        /// A buying offer
        Buy,
        /// A selling offer
        Sell,
    }

    impl OfferType {
        pub fn as_str(&self) -> &'static str {
            match self {
                OfferType::Buy => "Buy",
                OfferType::Sell => "Sell",
            }
        }
    }
}

/// Used as parameters to message functions
pub mod input {
    use super::*;

    /// Input to create_exchange function
    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct CreateExchangeRequest {
        /// Name of the token that is going to be traded through this order book model
        pub exchange_token: UniqueMultiTokenInfo,
        /// The optional unique id for the exchange token
        pub initial_amount: Balance,
        /// Name of the token of this first selling offer
        pub offer_token: UniqueMultiTokenInfo,
        /// Price per each exchange token of the Initial supply
        pub price: Balance,
    }

    /// Used in place_buying_offer and place_selling_offer function
    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PlaceOfferRequest {
        /// Id of the exchange
        pub exchange_id: ExchangeId,
        // TODO: what is this for?
        /// Address of the offer (this is not used)
        pub address: AccountId,
        /// Token of the offer
        pub offer_token: UniqueMultiTokenInfo,
        /// Amount of token for the order book
        pub amount: Balance,
        /// Price per each exchange token of the Initial supply
        pub price: Balance,
    }

    #[derive(Debug, Encode, Decode, Clone, Copy)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct OfferRequest {
        pub exchange_id: ExchangeId,
        pub offer_id: OfferId,
        pub address: AccountId,
        pub amount: Balance,
    }

    #[derive(Debug, Encode, Decode, Clone, Copy)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct CancelOfferRequest {
        pub exchange_id: ExchangeId,
        pub offer_id: OfferId,
    }
}

pub mod output {
    use super::*;

    /// Returned by get_exchange
    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ExchangeInfo {
        /// Unique ID that represents the exchange
        pub id: ExchangeId,
        /// CreatorAddress of the creator of the exchange
        pub creator_address: AccountId,
        /// ExchangeToken token that is going to be traded through this order book model
        pub exchange_token: UniqueMultiTokenInfo,
        /// InitialAmount of exchangeToken to sell
        pub initial_amount: Balance,
        /// Price per each exchange token
        pub price: Balance,
    }

    /// An offer for the exchange. Can be a buy or sell offer.
    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct OfferInfo {
        /// Id identifies the offer
        pub id: OfferId,
        /// ExchangeId identifies the initial offer
        pub exchange_id: ExchangeId,
        /// Type can be Buy or Sell
        pub r#type: OfferType,
        /// CreatorAddress of the creator of the offer
        pub creator_address: AccountId,
        /// Price in which the order was placed
        pub price: Balance,
        /// Amount of the token that the owner offers
        pub amount: Balance,
        /// Token of the offer
        pub offer_token: UniqueMultiTokenInfo,
    }
}

pub mod event_output {
    use super::*;

    /// Emitted when an exchange is created
    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct CreatedExchangeOutput {
        /// The ID of the exchange that was created
        pub exchange_id: Hash,
        /// The ID of the offer that was created
        pub offer_id: Hash,
    }

    /// An offer was placed
    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PlacedOfferOutput {
        /// The ID of the offer that was placed
        pub offer_id: OfferId,
    }

    /// An offer was placed
    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct CanceledOfferOutput {
        /// The ID of the offer that was canceled
        pub offer_id: OfferId,
    }
}
