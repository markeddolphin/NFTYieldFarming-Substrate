pub use event_output::*;
pub use input::*;
pub use other::*;
pub use output::*;
pub use storage::*;

use super::*;
use ink_prelude::{collections::BTreeMap, string::String, vec::Vec};
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

/// A share of a media that collabs can own
pub type CollabShare = u128;

/// Used by multiple modules
pub mod other {
    use super::*;

    /// The type of media
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout))]
    pub enum MediaType {
        Audio,
        Video,
        LiveAudio,
        LiveVideo,
        Blog,
        BlogSnap,
        DigitalArt,
        Claimable,
    }

    /// Info about the media viewing
    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout))]
    pub struct ViewInfo {
        pub viewing_type: ViewingType,
        /// ERC-20 token in which payment for opening Media is conducted
        pub viewing_token: AccountId,
        /// Price of opening Media
        pub price: Balance,
        /// A number between 0 and 100 (I think)
        pub sharing_percent: u128,
        pub is_streaming_live: bool,
        pub streaming_proportions: Vec<(String, Balance)>,
        pub token_reward: Vec<(AccountId, Balance)>,
        /// This appears to be the minimum amount of a token a user must have to open the media?
        pub token_entry: BTreeMap<AccountId, Balance>,
        /// Duration in case that the media viewing type is Dynamic
        pub duration: u64,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, PackedLayout, SpreadLayout)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout))]
    pub enum ViewingType {
        /// Dynamic means that Streaming will be created when opening Media
        Dynamic,
        /// Static means that immediate transfer will be executed when opening Media
        Fixed,
    }

    /// Struct for infos about the media's NFT infos
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout, Default)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout))]
    pub struct NftInfo {
        pub funding_token: AccountId,
        pub price: Balance,
    }
}

pub mod storage {
    use super::*;

    /// Unique identifier for Media
    pub type MediaId = erc721::TokenId;

    #[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout))]
    pub struct Media {
        /// Creator address of the creator
        pub creator: AccountId,
        /// Name of Media
        pub media_name: String,
        /// Symbol of the Media
        pub id: MediaId,
        /// Pods Address of the media's Pod
        pub pod_address: AccountId,
        /// Type of the Media,
        pub r#type: MediaType,
        /// Timestamp of the release of the media
        pub release_date: u64,
        /// View info of the media
        pub view_conditions: ViewInfo,
        /// NFT Conditions
        pub nft_conditions: NftInfo,
        /// Value that defines if the media is registered or not
        pub is_registered: bool,
        /// Value that defines if the media is uploaded or not
        pub is_uploaded: bool,
        /// Royalties that goes to the creators
        pub royalty: Balance,
    }

    // UpdateMediaProposal is the structure that holds the voters for a media update
    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout))]
    pub struct UpdateMediaProposal {
        /// The id of the media to change
        pub media_id: MediaId,
        /// The creator of the proposal
        pub requester_address: AccountId,
        /// The request being voted on
        pub update_request: UpdateMediaRequest,
        /// The current votes
        pub votes: BTreeMap<AccountId, bool>,
        /// The state of the proposal
        pub state: UpdateMediaProposalState,
        /// The minimum number of yes votes to accept
        pub min_approvals: u64,
        /// The maximum number of no votes to deny
        pub max_denials: u64,
        /// The amount of time the proposal is valid
        pub duration: u64,
        /// The time stamp the proposal was created
        pub date: u64,
    }

    impl UpdateMediaProposal {
        /// Returns the vote counts (yes, no)
        pub fn count_votes(&self) -> VoteCount {
            let mut yes_count = 0;
            let mut no_count = 0;
            for vote in self.votes.values() {
                if *vote {
                    yes_count += 1;
                } else {
                    no_count += 1;
                }
            }
            VoteCount { yes_count, no_count }
        }

        /// Is the time expired
        pub fn is_expired(&self, now: u64) -> bool { self.date + self.duration <= now }
    }

    /// Count of votes
    pub struct VoteCount {
        /// Number of yeses
        pub yes_count: u64,
        /// Number of nos
        pub no_count: u64,
    }

    /// Key for looking up an `UpdateMediaProposal`
    #[derive(Debug, Hash, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout))]
    pub struct ProposalKey {
        /// The media id
        pub media_id: MediaId,
        /// the creator of the proposal
        pub requester: AccountId,
    }

    /// These will be the new values for the media if the update request is approved
    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout))]
    pub struct UpdateMediaRequest {
        pub media_id: MediaId,
        pub creator_address: AccountId,
        pub media_name: String,
        pub r#type: MediaType,
        pub view_conditions: ViewInfo,
        pub nft_conditions: NftInfo,
        pub royalty: Balance,
        pub collabs: BTreeMap<AccountId, CollabShare>,
    }

    /// The state of an `UpdateMediaProposal`
    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout))]
    pub enum UpdateMediaProposalState {
        Pending,
        Accepted,
        Denied,
    }

    /// Unique identifier for MediaSharing
    pub type SharingId = u64;

    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout))]
    pub struct MediaSharing {
        pub media_id: MediaId,
        pub parent_id: Option<SharingId>,
        pub address: AccountId,
        pub id: SharingId,
    }
}

pub mod input {
    use super::*;

    #[derive(Debug, Clone, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct CreateMediaRequest {
        /// Address of the creator of the Media
        pub creator_address: AccountId,
        /// The name of the Media
        pub media_name: String,
        /// Pods Address of the media's Pod
        pub pod_address: AccountId,
        /// Type of the Media,
        pub r#type: MediaType,
        /// View info of the media
        pub view_conditions: ViewInfo,
        /// NFT Conditions
        pub nft_conditions: NftInfo,
        /// Royalties that goes to the creators
        pub royalty: Balance,
        /// Collaborators of the media + the allocation
        pub collabs: Option<BTreeMap<AccountId, CollabShare>>,
    }

    /// A vote on a proposal
    #[derive(Debug, Clone, Copy, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct UpdateMediaVote {
        /// The media id to vote on
        pub media_id: MediaId,
        /// the requester of the proposal
        pub requester_address: AccountId,
        /// The yes or no vote
        pub vote: bool,
    }

    /// Structure that allows a collab to fractionalise its sharing into one or more addresses
    #[derive(Debug, Clone, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct FractionaliseCollabRequest {
        pub media_id: MediaId,
        /// I think the number is a percentage between 0 and 100
        pub sharings: BTreeMap<AccountId, CollabShare>,
    }

    /// Used by open_media
    #[derive(Debug, Clone, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct OpenMediaRequest {
        pub media_id: MediaId,
        pub sharing_id: Option<SharingId>,
    }

    /// Used by close_media
    #[derive(Debug, Clone, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct CloseMediaRequest {
        pub media_id: MediaId,
    }


    #[derive(Debug, Clone, Copy, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ShareMediaRequest {
        pub media_id: MediaId,
        /// Id of last vertex of the sharing chain
        pub parent_id: Option<SharingId>,
    }

    /// Used with tipping a media
    #[derive(Debug, Clone, Copy, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct TipMediaRequest {
        /// The media id
        pub media_id: MediaId,
        /// Amount to tip
        pub amount: Balance,
        /// Account of the token to tip
        pub token: AccountId,
    }
}

pub mod output {
    use super::*;

    /// Information about a Media
    #[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout))]
    pub struct MediaInfo {
        /// Creator address of the creator
        pub creator: AccountId,
        /// Name of Media
        pub media_name: String,
        /// Symbol of the Media
        pub id: MediaId,
        /// Pods Address of the media's Pod
        pub pod_address: AccountId,
        /// Type of the Media,
        pub r#type: MediaType,
        /// Timestamp of the release of the media
        pub release_date: u64,
        /// View info of the media
        pub view_conditions: ViewInfo,
        /// NFT Conditions
        pub nft_conditions: NftInfo,
        /// Value that defines if the media is registered or not
        pub is_registered: bool,
        /// Value that defines if the media is uploaded or not
        pub is_uploaded: bool,
        /// Royalties that goes to the creators
        pub royalty: Balance,
        /// Collaborators of the media + the allocation
        pub collabs: BTreeMap<AccountId, CollabShare>,
    }

    impl From<MediaInfo> for Media {
        fn from(x: MediaInfo) -> Self {
            Self {
                creator: x.creator,
                media_name: x.media_name,
                id: x.id,
                pod_address: x.pod_address,
                r#type: x.r#type,
                release_date: x.release_date,
                view_conditions: x.view_conditions,
                nft_conditions: x.nft_conditions,
                is_registered: x.is_registered,
                is_uploaded: x.is_uploaded,
                royalty: x.royalty,
            }
        }
    }
}

pub mod event_output {
    use super::*;

    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct CreatedMediaOutput {
        /// The id of the media that was created
        pub media_id: MediaId,
    }

    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct SharedMediaOutput {
        /// The id of the SharingMedia
        pub sharing_id: SharingId,
    }
}
