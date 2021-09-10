#![feature(option_result_contains)]
#![feature(bool_to_option)]
#![feature(toowned_clone_into)]
#![cfg_attr(not(feature = "std"), no_std)]

mod constants;
mod errors;
mod models;

use ink_lang as ink;

pub use crate::claimable_media::ClaimableMedia;
pub use errors::{ProposeDistributionError, UpdateClaimableMediaError};

#[ink::contract]
mod claimable_media {
    use crate::{
        constants::WEEK,
        errors::{ProposeDistributionError, UpdateClaimableMediaError, ValidateDistributionError},
        models::{
            ClaimableMediaInfo, ClaimableMediaState, CreateClaimableMediaRequest, Distribution,
            DistributionProposalState,
        },
    };
    use ink_prelude::{collections::BTreeMap, string::String, vec::Vec};
    use ink_storage::collections::HashMap as StorageHashmap;
    use media::{
        models::{CreateMediaRequest, MediaType},
        MediaStorage as Media,
    };

    #[ink(storage)]
    pub struct ClaimableMedia {
        name: String,
        artists: StorageHashmap<AccountId, ()>,
        creator: AccountId,
        created_at: Timestamp,
        state: ClaimableMediaState,
        distributions: StorageHashmap<AccountId, Distribution>,
        media: Media,
        media_id: u64,
        erc1620: erc1620::Erc1620,
        erc20: erc20::Erc20,
    }

    #[ink(event)]
    pub struct DistributionProposed {
        account: AccountId,
    }

    impl ClaimableMedia {
        /// Creates a new claimable media, and an associated media object using the passed media contract.
        #[ink(constructor)]
        pub fn new(request: CreateClaimableMediaRequest) -> Self {
            let CreateClaimableMediaRequest { erc20, nft_info, name, view_info, artists, media, erc1620 } = request;
            let created_at = Self::env().block_timestamp();
            let creator = Self::env().caller();
            let contract_account_id = Self::env().account_id();
            let mut media = media;

            let mut collabs = BTreeMap::new();
            collabs.insert(contract_account_id, ::media::constants::COLLAB_SHARE_COUNT);

            let media_id = media
                .create_media(CreateMediaRequest {
                    creator_address: Self::env().caller(),
                    media_name: name.clone(),
                    pod_address: Self::env().account_id(),
                    r#type: MediaType::Audio,
                    view_conditions: view_info,
                    nft_conditions: nft_info,
                    royalty: 1,
                    collabs: Some(collabs),
                })
                .expect("unable to create media");

            let artists = artists.into_iter().map(|id| (id, ())).collect();

            Self {
                erc20,
                name,
                artists,
                creator,
                created_at,
                state: ClaimableMediaState::default(),
                distributions: Default::default(),
                media,
                media_id,
                erc1620,
            }
        }

        /// Change the artist of the claimable media. Artists are always
        /// added, but cannot be removed.
        ///
        /// # Restrictions
        ///
        /// May only be called by the creator of the claimable media.
        #[ink(message)]
        pub fn add_artists(&mut self, artists: Vec<AccountId>) -> Result<(), UpdateClaimableMediaError> {
            if self.creator != self.env().caller() {
                return Err(UpdateClaimableMediaError::Unauthorized);
            }

            self.artists.extend(artists.into_iter().map(|i| (i, ())));

            Ok(())
        }

        /// Change the state of the media.
        ///
        /// # Restrictions
        ///
        /// May only be called by the creator of the claimable media.
        #[ink(message)]
        pub fn set_state(&mut self, media_state: ClaimableMediaState) -> Result<(), UpdateClaimableMediaError> {
            if self.creator != self.env().caller() {
                return Err(UpdateClaimableMediaError::Unauthorized);
            }

            self.state = media_state;

            Ok(())
        }

        #[ink(message)]
        pub fn info(&self) -> ClaimableMediaInfo {
            let artists = self.artists.keys().cloned().collect();


            ClaimableMediaInfo {
                name: self.name.clone(),
                artists,
                creator: self.creator,
                created_at: self.created_at,
                state: self.state.clone(),
                media: self.media.clone(),
                media_id: self.media_id,
                erc1620: self.erc1620.clone(),
                erc20: self.erc20.clone(),
            }
        }

        /// Gets the proposed distribution of the claimable media.
        #[ink(message)]
        pub fn distribution(&self, proposer: AccountId) -> Option<Distribution> {
            self.distributions.get(&proposer).cloned()
        }

        /// Gets all proposed distributions for this claimable media.
        pub fn distributions(&self) -> BTreeMap<AccountId, Distribution> {
            let mut result = BTreeMap::new();
            for (k, v) in self.distributions.iter() {
                result.insert(*k, v.clone());
            }
            result
        }

        /// Adds a distribution proposal to the claimable media.
        ///
        /// # Restrictions
        ///
        /// May only be called by one of the artists.
        #[ink(message)]
        pub fn propose_distribution(
            &mut self,
            collabs: BTreeMap<AccountId, Balance>,
        ) -> Result<(), ProposeDistributionError> {
            let caller = self.env().caller();
            let now = self.env().block_timestamp();

            self.is_artist(caller).then_some(()).ok_or(ProposeDistributionError::Unauthorized)?;

            self.distributions.insert(caller, Distribution {
                collabs,
                validations: Default::default(),
                state: DistributionProposalState::Pending,
                created_at: now,
            });

            self.env().emit_event(DistributionProposed { account: caller });

            Ok(())
        }

        /// Votes and validates the claimable media. If all artists agreed to a distribution, that
        /// distribution is finalized. If one artists votes against the proposal, it is permanently
        /// denied
        ///
        /// # Restrictions
        ///
        /// May only be called by one of the artists.
        #[ink(message)]
        pub fn validate(&mut self, proposer: AccountId, accept: bool) -> Result<(), ValidateDistributionError> {
            let caller = self.env().caller();
            let now = self.env().block_timestamp();
            let contract_account_id = self.env().account_id();

            self.is_artist(caller).then_some(()).ok_or(ValidateDistributionError::Unauthorized)?;
            let mut distribution = self.distributions.get_mut(&proposer).ok_or(ValidateDistributionError::NotFound)?;

            if !distribution.state.is_pending() {
                return Err(ValidateDistributionError::NotPending);
            }

            distribution.validations.insert(caller, true);

            let time_since_proposed = now - distribution.created_at;

            if !accept || time_since_proposed > WEEK {
                distribution.state = DistributionProposalState::Denied;
                return Ok(());
            }

            let accepted =
                self.artists.keys().all(|account_id| Some(&true) == distribution.validations.get(account_id));

            if accepted {
                // by failing on transfers, we roll back the state, that's better than arriving at an
                // inconsistent state.
                self.erc1620.withdraw_from_all_streams().expect("withdrawing from own streams");
                let total = self.erc20.balance_of(contract_account_id);

                let creator_share = total / 1000;
                let royalties = total - creator_share;
                self.erc20.transfer(self.creator, creator_share).expect("transferring creator's share");

                for (artist, royalty) in distribute_shares(royalties, distribution.collabs.clone()) {
                    self.erc20.transfer(artist, royalty).expect("transferring royalties");
                }

                // There will be a remainder left in the medias account. This should be relatively
                // insignificant and will be divided during the next proposal.
            }

            Ok(())
        }

        fn is_artist(&self, account_id: AccountId) -> bool { self.artists.contains_key(&account_id) }
    }

    /// Uses euclidean division to distribute the royalties over the shares. Note that there will be
    /// a significant remainder in some cases, which can be handled by another distribution call.
    pub(crate) fn distribute_shares<T>(
        royalties: Balance,
        distribution: BTreeMap<T, Balance>,
    ) -> impl Iterator<Item = (T, Balance)> {
        let total_share_count = distribution.values().sum();
        let per_share = royalties.checked_div_euclid(total_share_count).unwrap();

        distribution.into_iter().map(move |(account, shares)| {
            let royalty = per_share.checked_mul(shares).unwrap();
            (account, royalty)
        })
    }
}


#[cfg(test)]
mod tests {
    use crate::claimable_media::distribute_shares;
    use ink_prelude::collections::BTreeMap;

    #[test]
    fn test_share_distribution_one() {
        let royalty = 1;
        let mut distribution = BTreeMap::new();
        distribution.insert((), 1);
        let (_, got) = distribute_shares(royalty, distribution).next().unwrap();
        assert_eq!(royalty, got)
    }

    #[test]
    fn test_share_distribution_three() {
        let royalty = 123456789;
        let mut distribution = BTreeMap::new();
        distribution.insert(1, 1);
        distribution.insert(2, 13);
        distribution.insert(3, 1802);
        let got: Vec<_> = distribute_shares(royalty, distribution).collect();
        assert_eq!(vec![(1, 67982), (2, 883766), (3, 122503564),], got)
    }
}
