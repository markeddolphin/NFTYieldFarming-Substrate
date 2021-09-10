use err_derive::Error;

use ink_prelude::string::String;

#[derive(Debug, Error, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
#[error(display = "action only allowed by the contract owner")]
pub struct OwnerError;

#[derive(Debug, Error, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
#[error(display = "action only allowed by registered and active oracles")]
pub struct OracleError;

#[derive(Debug, Error, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
#[error(display = "logic error (this indicates that something quite bad happened, such as overflows): {}", _0)]
pub struct MathError(String);

#[derive(Debug, Error, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum SubmitPriceError {
    #[error(display = "authorization error: {}", _0)]
    AuthzError(#[error(source)] OracleError),
}

#[derive(Debug, Error, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum GetPriceError {
    #[error(display = "pricing bucket not found")]
    BucketNotFound,
    #[error(display = "{}", _0)]
    MathError(MathError),
}

impl GetPriceError {
    pub fn math_error(msg: impl Into<String>) -> GetPriceError { GetPriceError::MathError(MathError(msg.into())) }
}

#[derive(Debug, Error, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RegisterOracleError {
    #[error(display = "account id is already a registered oracle")]
    AlreadyExists,
    #[error(display = "authorization error: {}", _0)]
    AuthzError(#[error(source)] OwnerError),
}

#[derive(Debug, Error, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum UpdateOracleStateError {
    #[error(display = "oracle not found")]
    DoesNotExist,
    #[error(display = "authorization error: {}", _0)]
    AuthzError(#[error(source)] OwnerError),
}

#[derive(Debug, Error, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ConvertError {
    #[error(display = "error obtaining token prices: {}", _0)]
    GetPriceError(#[error(source)] GetPriceError),

    #[error(display = "transfer error: {}", _0)]
    Erc20(#[error(source)] erc20::Error),

    #[error(display = "requested tickers were not associated with a token: {}", token)]
    TokenNotFound { token: String },

    #[error(display = "token value is zero (either an oracle has not registered the price, or it is too low to swap)")]
    TokenValueIsZero,

    #[error(display = "invalid price: price exceeded i128")]
    InvalidPrice,
}

impl ConvertError {
    pub fn token_not_found(token: impl Into<String>) -> ConvertError {
        ConvertError::TokenNotFound { token: token.into() }
    }
}
