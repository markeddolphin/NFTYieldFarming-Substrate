use err_derive::Error;
use scale::{Decode, Encode};

/// The Pod Media possible Errors.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Error)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    #[error(display = "funding closed")]
    FundingClosed,

    #[error(display = "media already registered")]
    MediaAlreadyRegistered,

    #[error(display = "authorization error")]
    MediaNotRegistered,

    #[error(display = "media not found")]
    MediaNotFound,

    #[error(display = "pod is not in investing state")]
    PodNotInInvestState,

    #[error(display = "erc20 error: {}", _0)]
    Erc20(#[error(source)] erc20::Error),

    #[error(display = "error creating media: {}", _0)]
    CreateMedia(#[error(source)] media::Error),

    #[error(display = "only the pod/media owner may perform this operation")]
    Unauthorized,

    #[error(display = "media's release date must be in the future")]
    ReleaseDateMustBeInFuture,
}

/// Errors encountered during the validation of a `CreateInvestingPodRequest`.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Error)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum InvestingPodValidationError {
    #[error(display = "funding date must be in future")]
    FundingDateMustBeInFuture,
    #[error(display = "pod must have at least one media")]
    PodMustHaveAtLeastOneMedia,
    #[error(display = "spread must be smaller than one")]
    SpreadMustBeSmallerThanOne,
    #[error(display = "funding token price must be greater than zero")]
    FundingTokenPriceCannotBeZero,
}
