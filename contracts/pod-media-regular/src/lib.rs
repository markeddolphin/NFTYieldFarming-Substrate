#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

pub mod errors;
pub mod models;

#[ink::contract]
mod pod_media_investing {
    use crate::models::{CreatePodRequest, RegisterMediaRequest};

    use crate::errors::Error;

    use media::models::MediaId;

    cfg_if::cfg_if! {
        if #[cfg(not(feature = "ink-as-dependency"))] {
            use ink_storage::collections::Vec as StorageVec;
            use media::MediaStorage;
            use crate::models::PodState;
        }
    }

    type Result<T> = core::result::Result<T, Error>;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Pod {
        creator: AccountId,
        media_ids: StorageVec<MediaId>,
        state: PodState,
        media: MediaStorage,
        created_at: Timestamp,
    }

    impl Pod {
        #[ink(constructor)]
        pub fn new(request: CreatePodRequest) -> Self {
            let now = Self::env().block_timestamp();

            let pod_address = Self::env().account_id();
            let caller = Self::env().caller();

            let mut media_contract = request.media_contract;

            let media_ids: StorageVec<u64> = request
                .medias
                .into_iter()
                .map(|media| {
                    let media = media.into_media_request(caller, pod_address);
                    media_contract.create_media(media).unwrap()
                })
                .collect();

            Self {
                creator: Self::env().caller(),
                media: media_contract,
                created_at: now,
                state: PodState { registered_media: 0, total_media: media_ids.len() },
                media_ids,
            }
        }

        /// Sets the parameters of a media object. May only be called once, which then registers the
        /// media.
        #[ink(message)]
        pub fn register_media(&mut self, request: RegisterMediaRequest) -> Result<()> {
            let now = self.env().block_timestamp();
            if request.release_date < now {
                return Err(Error::ReleaseDateMustBeInFuture);
            }

            let caller = self.env().caller();

            // get the media by id
            let mut media = self.media.get_media(request.media_id).ok_or(Error::MediaNotFound)?;
            // check that the caller owns the media.
            if media.creator != caller {
                return Err(Error::Unauthorized);
            }

            if media.is_registered {
                return Err(Error::MediaAlreadyRegistered);
            }
            media.release_date = request.release_date;
            media.is_registered = true;
            media.view_conditions.viewing_type = request.payment_type;
            media.view_conditions.price = request.price;
            media.view_conditions.viewing_token = request.funding_token;

            self.media.update_collabs(media.id, request.collabs)?;
            self.media.update_media(media.into())?;
            self.state.increment_registered_media();
            Ok(())
        }

        /// Sets the media.is_uploaded field to true.
        ///
        /// # Restrictions
        ///
        /// * May only be called by the media creator.
        /// * Only registered media may be uploaded.
        #[ink(message)]
        pub fn upload_media(&mut self, media_id: MediaId) -> Result<()> {
            let mut media = self.media.get_media(media_id).ok_or(Error::MediaNotFound)?;

            if !media.is_registered {
                return Err(Error::MediaNotRegistered);
            }

            if media.creator != self.env().caller() {
                return Err(Error::Unauthorized);
            }

            media.is_uploaded = true;
            self.media.update_media(media.into())?;
            Ok(())
        }

        /// AccountId of the pod creator.
        #[ink(message)]
        pub fn creator(&self) -> AccountId { self.creator }
    }
}
