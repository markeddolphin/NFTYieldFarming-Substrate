use err_derive::Error;

#[derive(Debug, Error, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum UpdateClaimableMediaError {
    #[error(display = "only the creator may update the claimable media")]
    Unauthorized,
}

#[derive(Debug, Error, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ProposeDistributionError {
    #[error(display = "only artists part of the media may propose a distribution")]
    Unauthorized,
}


#[derive(Debug, Error, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ValidateDistributionError {
    #[error(display = "only artists part of the media may propose a distribution")]
    Unauthorized,
    #[error(display = "only distributions in pending state may be validated")]
    NotPending,
    #[error(display = "distribution not found")]
    NotFound,
}
