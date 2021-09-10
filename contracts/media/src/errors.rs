use scale::{Decode, Encode};

/// The Error type for this crate
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, err_derive::Error)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    /// The sum of the collab shares is invalid
    #[error(display = "The sum of the collab shares is invalid")]
    InvalidSumOfCollabShares,
    /// One or more collab shares is out of range
    #[error(display = "One or more collab shares is out of range")]
    CollabShareOutOfRange,
    /// A math operation overflowed
    #[error(display = "A math operation overflowed")]
    Overflow,
    /// The owner is required for this operation
    #[error(display = "The owner is required for this operation")]
    OwnerRequired,
    /// The collaborators do not exist
    #[error(display = "The collaborators do not exist")]
    CollaboratorsNotFound,
    /// The media does not exist
    #[error(display = "The media does not exist")]
    MediaNotFound,
    /// The media sharing parent id is invalid
    #[error(display = "The media sharing parent id is invalid")]
    InvalidMediaSharingParentId,
    /// The media sharing parent does not exist
    #[error(display = "The media sharing parent does not exist")]
    MediaSharingParentNotFound,
    /// The media sharing does not exist
    #[error(display = "The media sharing does not exist")]
    MediaSharingNotFound,
    /// The community was not found for the proposal
    #[error(display = "The community was not found for the proposal")]
    CommunityNotFound,
    /// The proposal could not be found
    #[error(display = "The proposal could not be found")]
    ProposalNotFound,
    /// The account is required to be a collaborator
    #[error(display = "The account is required to be a collaborator")]
    RequiresCollaborator,
    /// The account is mot allowed to vote on this proposal
    #[error(display = "The account is mot allowed to vote on this proposal")]
    VoteNotAllowed,
    /// The balance is insufficient
    #[error(display = "The balance is insufficient")]
    InsufficientBalance,
    /// An ERC-1620 error occurred
    #[error(display = "An Erc1620 error occurred: {}", _0)]
    Erc1620(#[source] erc1620::Error),
    /// An ERC-20 error occurred
    #[error(display = "An Erc20 error occurred: {}", _0)]
    Erc20(#[source] erc20::Error),
    /// An ERC-721 error occurred
    #[error(display = "An Erc721 error occurred: {}", _0)]
    Erc721(#[source] erc721::Error),

    /// Message is only callable by the media's pod address.
    #[error(display = "only callable by the pod address contract")]
    PodAddressRequired,
}
