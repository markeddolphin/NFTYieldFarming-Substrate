use err_derive::Error;
use scale::{Decode, Encode};

/// The Pod Media possible Errors.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Error)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    #[error(display = "media already registered")]
    MediaAlreadyRegistered,
    //
    #[error(display = "authorization error")]
    MediaNotRegistered,

    #[error(display = "media not found")]
    MediaNotFound,

    #[error(display = "error creating media: {}", _0)]
    CreateMedia(#[error(source)] media::Error),

    #[error(display = "only the pod/media owner may perform this operation")]
    Unauthorized,

    #[error(display = "media's release date must be in the future")]
    ReleaseDateMustBeInFuture,
}
