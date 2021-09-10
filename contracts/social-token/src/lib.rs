#![cfg_attr(not(feature = "std"), no_std)]

mod amm;
mod contract;

use ink_env::DefaultEnvironment;
use ink_lang as ink;
use ink_prelude::string::String;
use rust_decimal_macros::dec;
use scale::{Decode, Encode};

/// The Error type for this crate
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, err_derive::Error)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    /// Returned if not enough balance to fulfill a request is available.
    #[error(display = "Returned if not enough balance to fulfill a request is available.")]
    InsufficientBalance,
    /// Not enough balance in initial supply
    #[error(display = "Insufficient initial supply balance")]
    InsufficientInitialSupplyBalance,
    /// Not enough balance in trading fee
    #[error(display = "Insufficient trading fee balance")]
    InsufficientTradingFeeBalance,
    #[error(display = "Missing permission to perform this operation")]
    InsufficientAccess,
    /// An ERC-20 error occurred
    #[error(display = "An Erc20 error occured: _0")]
    Erc20(#[source] erc20::Error),
}

/// The Result type for this crate
pub type Result<T> = core::result::Result<T, Error>;
