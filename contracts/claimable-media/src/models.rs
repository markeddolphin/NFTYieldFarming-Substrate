use ink_prelude::{collections::BTreeMap, string::String, vec::Vec};
use ink_storage::traits::{PackedLayout, SpreadLayout};
use media::{
    models::{NftInfo, ViewInfo},
    MediaStorage as Media,
};
pub type Balance = <ink_env::DefaultEnvironment as ink_env::Environment>::Balance;
pub type AccountId = <ink_env::DefaultEnvironment as ink_env::Environment>::AccountId;
pub type Timestamp = <ink_env::DefaultEnvironment as ink_env::Environment>::Timestamp;


#[derive(Debug, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout))]
pub struct CreateClaimableMediaRequest {
    pub name: String,
    pub artists: Vec<AccountId>,
    pub media: Media,
    pub view_info: ViewInfo,
    pub nft_info: NftInfo,
    pub erc1620: erc1620::Erc1620,
    pub erc20: erc20::Erc20,
}

#[derive(Debug, Clone, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout))]
pub struct ClaimableMediaInfo {
    pub name: String,
    pub artists: Vec<AccountId>,
    pub creator: AccountId,
    pub created_at: Timestamp,
    pub state: ClaimableMediaState,
    pub media: Media,
    pub media_id: u64,
    pub erc1620: erc1620::Erc1620,
    pub erc20: erc20::Erc20,
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout))]
pub enum ClaimableMediaState {
    Claimed,
    Verified,
    NotClaimed,
}

impl Default for ClaimableMediaState {
    fn default() -> Self { Self::NotClaimed }
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout))]
pub enum DistributionProposalState {
    Accepted,
    Denied,
    Pending,
}

impl DistributionProposalState {
    pub fn is_pending(&self) -> bool { matches!(self, DistributionProposalState::Pending) }
}

impl Default for DistributionProposalState {
    fn default() -> Self { Self::Pending }
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout))]
pub struct Distribution {
    pub collabs: BTreeMap<AccountId, Balance>,
    pub validations: BTreeMap<AccountId, bool>,
    pub state: DistributionProposalState,
    pub created_at: Timestamp,
}
