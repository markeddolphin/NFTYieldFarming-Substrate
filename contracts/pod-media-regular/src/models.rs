use ink_prelude::{collections::BTreeMap, string::String, vec::Vec};
use ink_storage::traits::{PackedLayout, SpreadLayout};
use media::{
    models::{CollabShare, MediaId, MediaType, NftInfo, ViewInfo, ViewingType},
    MediaStorage,
};


pub type Balance = <ink_env::DefaultEnvironment as ink_env::Environment>::Balance;
pub type AccountId = <ink_env::DefaultEnvironment as ink_env::Environment>::AccountId;
pub type Timestamp = <ink_env::DefaultEnvironment as ink_env::Environment>::Timestamp;
pub type Hash = <ink_env::DefaultEnvironment as ink_env::Environment>::Hash;

#[derive(Debug, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
pub struct CreatePodRequest {
    pub erc20_code_hash: Hash,
    pub endowment: Balance,
    pub media_contract: MediaStorage,
    pub medias: Vec<CreateMediaRequest>,
}

pub type Collabs = BTreeMap<AccountId, CollabShare>;

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout))]
pub struct CreateMediaRequest {
    pub name: String,
    /// Type of the Media,
    pub r#type: MediaType,
    /// View info of the media
    pub view_conditions: ViewInfo,
    /// NFT Conditions
    pub nft_conditions: NftInfo,
    /// Royalties that goes to the creators
    pub royalty: Balance,
    /// Collaborators of the media + the allocation
    pub collabs: Collabs,
}

impl CreateMediaRequest {
    pub fn into_media_request(
        self,
        creator_address: AccountId,
        pod_address: AccountId,
    ) -> ::media::models::CreateMediaRequest {
        ::media::models::CreateMediaRequest {
            creator_address,
            pod_address,
            media_name: self.name,
            r#type: self.r#type,
            view_conditions: self.view_conditions,
            nft_conditions: self.nft_conditions,
            royalty: self.royalty,
            collabs: Some(self.collabs),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout))]
pub struct PodState {
    pub registered_media: u32,
    pub total_media: u32,
}

impl PodState {
    pub fn increment_registered_media(&mut self) {
        self.registered_media += 1;
        assert!(self.registered_media <= self.total_media, "registered media cannot exceed total media")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout))]
pub struct RegisterMediaRequest {
    pub media_id: MediaId,
    pub funding_token: AccountId,
    pub price: Balance,
    pub release_date: Timestamp,
    pub payment_type: ViewingType,
    pub royalty: Balance,
    pub collabs: Collabs,
}
