#![cfg_attr(not(feature = "std"), no_std)]
use ink_lang as ink;

pub mod constants;
pub mod errors;
pub mod models;

pub use crate::media::{MediaStorage, Result};
pub use errors::Error;

use contract_utils::env_exports::*;

#[ink::contract]
mod media {
    use super::*;
    use crate::{errors::Error, models::*};
    use ink_prelude::collections::BTreeMap;
    use ink_storage::collections::HashMap;

    cfg_if::cfg_if! {
        if #[cfg(not(feature = "ink-as-dependency"))] {
            use ink_env::call::FromAccountId;
            use core::convert::{TryFrom, TryInto};
            use erc20::Erc20;
            use ink_prelude::{vec::Vec};
            use ink_storage::{Lazy};
        }
    }


    #[ink(storage)]
    pub struct MediaStorage {
        /// Owner of the contract
        owner: AccountId,
        /// The next `SharingId` to use when a media is shared
        next_sharing_id: Lazy<SharingId>,

        /// The ERC-721 contract used to mint media NFTs
        erc721: erc721::Erc721,
        /// The ERC-1620 contract used for streaming payments
        erc1620: erc1620::Erc1620,

        // media
        /// Medias by media id
        medias_by_id: HashMap<MediaId, Media>,
        /// Collaborators by media id
        collaborators_by_media_id: HashMap<MediaId, BTreeMap<AccountId, CollabShare>>,
        /// The sharings by id
        media_sharings_by_id: HashMap<SharingId, MediaSharing>,
        /// The streams being used by the media id
        streams_by_media_id: HashMap<MediaId, Vec<erc1620::StreamId>>,

        // proposals
        /// The current media proposals being voted on
        proposals_by_key: HashMap<ProposalKey, UpdateMediaProposal>,
        /// The communities for each proposal
        communities_by_proposal_key: HashMap<ProposalKey, BTreeMap<AccountId, ()>>,
    }

    /// Media result type.
    pub type Result<T> = core::result::Result<T, Error>;

    // Events

    /// Emitted when media is created
    #[ink(event)]
    #[derive(derive_new::new)]
    pub struct CreatedMedia {
        /// Ouput of the event
        pub output: CreatedMediaOutput,
    }

    #[ink(event)]
    #[derive(derive_new::new)]
    pub struct SharedMedia {
        /// Ouput of the event
        pub output: SharedMediaOutput,
    }

    impl MediaStorage {
        /// Create a new contract.
        #[allow(clippy::new_without_default)]
        #[ink(constructor)]
        pub fn new(erc1620_account_id: erc1620::Erc1620, erc721_account_id: erc721::Erc721) -> Self {
            let contract_owner = Self::env().caller();
            Self {
                owner: contract_owner,
                next_sharing_id: Default::default(),
                erc1620: erc1620_account_id,
                erc721: erc721_account_id,
                medias_by_id: Default::default(),
                collaborators_by_media_id: Default::default(),
                proposals_by_key: Default::default(),
                communities_by_proposal_key: Default::default(),
                media_sharings_by_id: Default::default(),
                streams_by_media_id: Default::default(),
            }
        }

        /// Creates a new Media and mints the NFT token for it.
        /// ### Arguments
        /// * creator_address - Address of the creator of the Media
        /// * media_name - The name of the Media
        /// * pod_address - Pods Address of the media's Pod
        /// * type - Type of the Media,
        /// * view_conditions - View info of the media
        /// * nft_conditions - NFT Conditions
        /// * royalty - Royalties that goes to the creators
        /// * collabs - Collaborators of the media + the allocation
        #[ink(message)]
        pub fn create_media(&mut self, input: CreateMediaRequest) -> Result<MediaId> {
            let caller = self.env().caller();

            // mint nft token
            let media_id = self.erc721.mint(caller)?;

            // update storage
            self.medias_by_id.insert(media_id, Media {
                creator: input.creator_address,
                media_name: input.media_name,
                id: media_id,
                pod_address: input.pod_address,
                r#type: input.r#type,
                release_date: self.env().block_timestamp(),
                view_conditions: input.view_conditions.clone(),
                nft_conditions: input.nft_conditions.clone(),
                is_registered: false,
                is_uploaded: false,
                royalty: input.royalty,
            });

            if let Some(collabs) = input.collabs {
                self.collaborators_by_media_id.insert(media_id, collabs);
            } else {
                self.collaborators_by_media_id.insert(media_id, Default::default());
            };

            self.env().emit_event(CreatedMedia::new(CreatedMediaOutput { media_id }));

            Ok(media_id)
        }

        /// Gets the media from the `media_id`
        #[ink(message)]
        pub fn get_media(&self, id: MediaId) -> Option<MediaInfo> {
            self.collaborators_by_media_id.get(&id).and_then(|collabs| {
                self.medias_by_id.get(&id).map(|x| MediaInfo {
                    creator: x.creator,
                    media_name: x.media_name.clone(),
                    id,
                    pod_address: x.pod_address,
                    r#type: x.r#type,
                    release_date: x.release_date,
                    view_conditions: x.view_conditions.clone(),
                    nft_conditions: x.nft_conditions,
                    is_registered: x.is_registered,
                    is_uploaded: x.is_uploaded,
                    royalty: x.royalty,
                    collabs: collabs.clone(),
                })
            })
        }

        /// Creates a proposal to update a `Media`
        /// ### Arguments
        /// See arguments for `create_media`
        #[ink(message)]
        pub fn create_update_media_proposal(&mut self, request: UpdateMediaRequest) -> Result<()> {
            let caller = self.env().caller();

            // make sure the requester is a collaborator
            let collaborators =
                self.collaborators_by_media_id.get(&request.media_id).ok_or(Error::CollaboratorsNotFound)?;

            if !collaborators.contains_key(&caller) {
                return Err(Error::RequiresCollaborator);
            }

            let media = self.medias_by_id.get(&request.media_id).ok_or(Error::MediaNotFound)?;

            // store the proposal
            let key = ProposalKey { media_id: media.id, requester: caller };
            self.proposals_by_key.insert(key, UpdateMediaProposal {
                media_id: media.id,
                requester_address: caller,
                update_request: request,
                votes: Default::default(),
                state: UpdateMediaProposalState::Pending,
                min_approvals: collaborators.len().try_into().expect("overflow"),
                max_denials: 1,
                duration: constants::UPDATE_MEDIA_PROPOSAL_DURATION,
                date: self.env().block_timestamp(),
            });
            self.communities_by_proposal_key.insert(key, collaborators.iter().map(|(k, _)| (*k, ())).collect());
            Ok(())
        }

        /// Vote on an `UpdateMediaProposal`
        /// ### Arguments
        /// * media_id - the id of the media
        /// * requester_address - the address of the user that made the request being voting on
        /// * vote - yes or no vote
        #[ink(message)]
        pub fn vote_media_update_proposal(&mut self, vote: UpdateMediaVote) -> Result<()> {
            let caller = self.env().caller();
            let key = ProposalKey { media_id: vote.media_id, requester: vote.requester_address };
            let now = self.env().block_timestamp();

            // get and validate data
            let proposal = self.proposals_by_key.get_mut(&key).ok_or(Error::ProposalNotFound)?;
            let community = self.communities_by_proposal_key.get(&key).ok_or(Error::ProposalNotFound)?;
            if !community.contains_key(&caller) {
                return Err(Error::VoteNotAllowed);
            }

            // add the vote and count them
            proposal.votes.insert(caller, vote.vote);
            let VoteCount { yes_count, no_count } = proposal.count_votes();

            // remove the proposal if expired or denied
            if proposal.is_expired(now) || no_count >= proposal.max_denials {
                self.proposals_by_key.take(&key);
                self.communities_by_proposal_key.take(&key);
                return Ok(());
            }

            if yes_count >= proposal.min_approvals {
                // remove the proposal
                let proposal = self.proposals_by_key.take(&key).ok_or(Error::ProposalNotFound)?;
                self.communities_by_proposal_key.take(&key).ok_or(Error::CommunityNotFound)?;
                let request = proposal.update_request;

                // update the media
                self.collaborators_by_media_id.insert(vote.media_id, request.collabs);

                let mut media = self.medias_by_id.get_mut(&vote.media_id).ok_or(Error::MediaNotFound)?;
                media.creator = request.creator_address;
                media.media_name = request.media_name;
                media.nft_conditions = request.nft_conditions;
                media.royalty = request.royalty;
                media.r#type = request.r#type;
                media.view_conditions = request.view_conditions;
            }
            Ok(())
        }

        /// Allows a collab to fractionalise its sharing into one or more addresses
        #[ink(message)]
        pub fn fractionalise_media_collab(&mut self, request: FractionaliseCollabRequest) -> Result<()> {
            // NOTE: it seems dangerous that this is updated without a vote, but collabs are also changed through vote
            // in vote_media_update_proposal

            let caller = self.env().caller();
            let collabs =
                self.collaborators_by_media_id.get_mut(&request.media_id).ok_or(Error::CollaboratorsNotFound)?;

            let current_share = collabs.remove(&caller).ok_or(Error::RequiresCollaborator)?;
            for (address, share) in request.sharings {
                collabs.insert(address, current_share.checked_mul(share).ok_or(Error::Overflow)?);
            }
            Ok(())
        }

        /// Replaces the data for the Media. Only callable by the medias `pod_address`.
        #[ink(message)]
        pub fn update_media(&mut self, media: Media) -> Result<Media> {
            let caller = self.env().caller();

            let stored = self.medias_by_id.get(&media.id).ok_or(Error::MediaNotFound)?;
            if stored.pod_address != caller {
                return Err(Error::PodAddressRequired);
            }
            // since we already checked if the media exists, this unwrap will not panic.
            Ok(self.medias_by_id.insert(media.id, media).unwrap())
        }

        /// Replaces the data for the Media. Only callable by the medias `pod_address`.
        #[ink(message)]
        pub fn update_collabs(&mut self, id: MediaId, collabs: BTreeMap<AccountId, CollabShare>) -> Result<()> {
            let caller = self.env().caller();
            let stored = self.medias_by_id.get(&id).ok_or(Error::MediaNotFound)?;
            if stored.pod_address != caller {
                return Err(Error::PodAddressRequired);
            }

            self.collaborators_by_media_id.insert(id, collabs.into_iter().collect());
            Ok(())
        }

        /// Called when a Media is being viewed
        /// ### Arguments
        /// * media_id	- the media id
        /// * sharing_id - the sharing id
        #[ink(message)]
        pub fn open_media(&mut self, request: OpenMediaRequest) -> Result<()> {
            let media = self.medias_by_id.get(&request.media_id).ok_or(Error::MediaNotFound)?;
            let caller = self.env().caller();

            // get total payment
            let mut payment_amount = media.view_conditions.price;

            // NOTE: this code looks suspicious. Why does payment become zero if a balance exists?
            // check if user accomplish entry token conditions
            for (token_account, requested) in &media.view_conditions.token_entry {
                if Erc20::from_account_id(*token_account).balance_of(caller) >= *requested {
                    payment_amount = 0;
                    break;
                }
            }

            // if payment is needed
            if payment_amount > 0 {
                let mut viewing_token = Erc20::from_account_id(media.view_conditions.viewing_token);

                // get the account that will be used to pay
                let (payment_account, balance) = {
                    let reward_account = contract_utils::get_reward_account_id(self.env(), caller);
                    let balance = viewing_token.balance_of(reward_account);
                    if balance >= payment_amount {
                        (reward_account, balance)
                    } else {
                        (caller, viewing_token.balance_of(caller))
                    }
                };

                // make sure the account has enough funds
                if balance < payment_amount {
                    return Err(Error::InsufficientBalance);
                }

                // calculate sharing fees
                let (shared, mut payments) = {
                    if let Some(sharing_id) = request.sharing_id {
                        self.get_sharing_proportions(
                            &media.view_conditions,
                            payment_amount,
                            sharing_id,
                            constants::GET_SHARING_PROPORTIONS_DEPTH,
                        )
                    } else {
                        (0, HashMap::new())
                    }
                };

                // calculate royalty fees
                let collabs =
                    self.collaborators_by_media_id.get(&request.media_id).ok_or(Error::CollaboratorsNotFound)?;
                let fee = utils::get_royalties(payment_amount, media.royalty, collabs, &mut payments);

                // calculate owners profit
                utils::get_owners_profit(payment_amount - shared - fee, collabs, &mut payments);

                // make sure caller does not pay self
                payments.take(&caller);

                match media.view_conditions.viewing_type {
                    // stream the transfers over time
                    ViewingType::Dynamic => {
                        // create a stream for each receiver
                        let now = self.env().block_timestamp();
                        for (receiver, balance) in payments.into_iter() {
                            self.erc1620.create_stream(
                                *receiver,
                                *balance,
                                media.view_conditions.viewing_token,
                                now,
                                now + media.view_conditions.duration,
                            )?;
                        }
                    }
                    // make the transfers immediately
                    ViewingType::Fixed => {
                        for (receiver, balance) in payments.iter() {
                            viewing_token.transfer_from(payment_account, *receiver, *balance)?;
                        }
                    }
                }
            }

            // send token rewards
            todo!("send reward to pod-media")
            // for (token_account, reward) in &media.view_conditions.token_reward {
            //     let token = Erc20::from_account_id(*token_account);
            //     if token.balance_of(media.pod_address) >= reward {
            //         token.transfer_from(media.pod_address, caller, reward);
            //     }
            //     // seems weird it does nothing if the balance is not enough to send the reward
            // }

            // Ok(())
        }

        /// Stop the streams used by `media_id` if they exist
        #[ink(message)]
        pub fn close_media(&mut self, media_id: MediaId) -> Result<()> {
            if let Some(stream_ids) = self.streams_by_media_id.take(&media_id) {
                for stream_id in stream_ids {
                    self.erc1620.cancel_stream(stream_id)?;
                }
            }

            Ok(())
        }

        /// Validates the request and generates a `SharingId`
        /// * parent_id - Id of last vertex of the sharing chain
        /// * media_id - Symbol of the Media
        #[ink(message)]
        pub fn share_media(&mut self, request: ShareMediaRequest) -> Result<SharingId> {
            let caller = self.env().caller();

            // validate the parent
            if let Some(parent_id) = request.parent_id {
                let parent = self.media_sharings_by_id.get(&parent_id).ok_or(Error::MediaSharingParentNotFound)?;
                if parent.media_id != request.media_id || parent.address != caller {
                    return Err(Error::InvalidMediaSharingParentId);
                }
            }

            // generate the sharing id and store it
            let sharing_id = self.increment_next_sharing_id();
            self.media_sharings_by_id.insert(sharing_id, MediaSharing {
                media_id: request.media_id,
                parent_id: request.parent_id,
                address: caller,
                id: sharing_id,
            });

            self.env().emit_event(SharedMedia::new(SharedMediaOutput { sharing_id }));

            Ok(sharing_id)
        }

        /// Tip the media
        /// * media_id - the media id
        /// * amount - amount of token to tip
        /// * token	- The AccountId of the token to tip
        #[ink(message)]
        pub fn tip_media(&self, request: TipMediaRequest) -> Result<()> {
            let media = self.medias_by_id.get(&request.media_id).ok_or(Error::MediaNotFound)?;
            let collabs = self.collaborators_by_media_id.get(&request.media_id).ok_or(Error::CollaboratorsNotFound)?;
            let caller = self.env().caller();
            let mut token = Erc20::from_account_id(request.token);
            let balance = token.balance_of(caller);
            let payment_amount = request.amount;
            if balance < request.amount {
                return Err(Error::InsufficientBalance);
            }

            // NOTE: this is the same code used in open_media
            // calculate royalty fees
            let mut payments = HashMap::new();
            let fee = utils::get_royalties(payment_amount, media.royalty, collabs, &mut payments);

            // calculate owners profit
            utils::get_owners_profit(payment_amount - fee, collabs, &mut payments);

            for (receiver, balance) in payments.into_iter() {
                token.transfer_from(caller, *receiver, *balance)?;
            }
            Ok(())
        }
    }

    #[ink(impl)]
    impl MediaStorage {
        fn increment_next_sharing_id(&mut self) -> MediaId {
            let value = *self.next_sharing_id;
            *self.next_sharing_id += 1;
            value
        }

        /// compute the sharing chain to rollback
        fn get_sharing_chain(&self, mut sharing_id: SharingId, mut depth: usize) -> Vec<AccountId> {
            let mut accounts = Vec::with_capacity(depth);

            while depth > 0 {
                if let Some(data) = self.media_sharings_by_id.get(&sharing_id) {
                    accounts.push(data.address);
                    if let Some(parent_id) = data.parent_id {
                        sharing_id = parent_id;
                        depth -= 1;
                    } else {
                        break;
                    }
                }
            }

            accounts
        }

        /// Compute sharing proportions to be distributed between people who shared the media.
        fn get_sharing_proportions(
            &self,
            info: &ViewInfo,
            price: Balance,
            sharing_id: SharingId,
            depth: usize,
        ) -> (Balance, HashMap<AccountId, Balance>) {
            let chain = self.get_sharing_chain(sharing_id, depth);
            let total = chain.len();
            let factor = Self::get_sharing_division_factor(total.try_into().expect("overflow"));

            let mut balances = HashMap::new();
            let shared = price * info.sharing_percent / 100;

            if info.sharing_percent > 0 {
                for (i, address) in chain.into_iter().enumerate() {
                    let value = u128::try_from(total - i).expect("overflow");
                    balances.insert(address, value / factor * shared);
                }
            }
            (shared, balances)
        }

        /// not sure what this does
        fn get_sharing_division_factor(n: u128) -> u128 { n * (n + 1) / 2 }
    }

    /// utility functions
    #[allow(dead_code)]
    mod utils {
        use super::*;

        /// compute profit to distribute to media owners
        pub fn get_owners_profit(
            payment: Balance,
            collabs: &BTreeMap<AccountId, CollabShare>,
            into: &mut HashMap<AccountId, Balance>,
        ) {
            distribute_amount(payment, collabs, into)
        }

        /// compute royalty that goes to artists
        pub fn get_royalties(
            amount: Balance,
            royalty: Balance,
            collabs: &BTreeMap<AccountId, CollabShare>,
            into: &mut HashMap<AccountId, Balance>,
        ) -> Balance {
            let fee = amount * royalty;
            distribute_amount(fee, collabs, into);
            fee
        }

        /// Multiplies amount * share for each item and adds or inserts into `into`
        pub fn distribute_amount<'a>(
            amount: Balance,
            receivers: impl IntoIterator<Item = (&'a AccountId, &'a CollabShare)>,
            into: &mut HashMap<AccountId, Balance>,
        ) {
            for (account, share) in receivers.into_iter() {
                // TODO: is this math correct
                let value = amount * (share / constants::COLLAB_SHARE_COUNT);
                into.entry(*account).and_modify(|x| *x += value).or_insert(value);
            }
        }
    }
}
