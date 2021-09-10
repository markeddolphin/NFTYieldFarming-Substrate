#![cfg_attr(not(feature = "std"), no_std)]

mod contract;
mod model;

use scale::{Decode, Encode};

/// The Error type for this crate
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, err_derive::Error)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    /// The exchange does not exist for the given id
    #[error(display = "The exchange does not exist for the given id")]
    ExchangeNotFound,
    /// The offer does not exist for the given id
    #[error(display = "The offer does not exist for the given id")]
    OfferNotFound,
    /// Offer type does not match requested operation
    #[error(display = "Cannot buy/sell from incompatible offer")]
    OfferTypeMismatch,
    /// Balance is not enough for exchange
    #[error(display = "Balance is not enough for exchange")]
    InsufficientBalance,
    /// An ERC-20 error occurred
    #[error(display = "erc20 error {}", _0)]
    MultiToken(#[source] multi_token::Error),
}

/// The Result type for this crate
pub type Result<T> = core::result::Result<T, Error>;
