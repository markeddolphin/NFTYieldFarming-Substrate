use erc20::Erc20;

use ink_prelude::{string::String, vec::Vec};
use ink_storage::traits::{PackedLayout, SpreadLayout};
use media::MediaStorage;

use crate::errors::InvestingPodValidationError;
use num_traits::Zero;
pub use pod_media_regular::models::{Collabs, CreateMediaRequest, RegisterMediaRequest};

pub type Balance = <ink_env::DefaultEnvironment as ink_env::Environment>::Balance;
pub type AccountId = <ink_env::DefaultEnvironment as ink_env::Environment>::AccountId;
pub type Timestamp = <ink_env::DefaultEnvironment as ink_env::Environment>::Timestamp;
pub type Hash = <ink_env::DefaultEnvironment as ink_env::Environment>::Hash;

#[derive(Debug, Clone, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
pub struct CreateInvestingPodRequest {
    /// Ticker of the pod token.
    pub pod_token_symbol: String,
    /// Full name of the pod token.
    pub pod_token_name: String,
    /// Erc20 token used as payment during investment period.
    pub funding_token: Erc20,
    /// Price per pod token during during investment period.
    pub funding_token_price: Balance,
    /// Funding target that needs to be reached for the pod to enter `Trading` state.
    pub funding_target: Balance,
    /// Mathematical curve of the AMM created when the pod reaches investing state.
    pub amm: amm::Curve,
    /// Spread of the AMM created when the pod reaches investing state.
    pub spread: u32,
    /// Maximum price used by the AMM.
    pub max_price: Balance,
    /// Maximum supply used by the AMM.
    pub max_supply: Balance,
    /// Date after which the funding period of a pod closes, regardless of reaching the funding goal.
    pub funding_date: Timestamp,
    /// Hash uses of the erc20 contract deployed for the pod token. Caller must ensure that the wasm
    /// for the contract has already been uploaded.
    pub erc20_code_hash: Hash,
    /// Media contract used to create medias.
    pub media_contract: MediaStorage,
    /// Medias to be created upon pod instantiation.
    pub medias: Vec<CreateMediaRequest>,
}

impl CreateInvestingPodRequest {
    pub fn validate(&self, now: Timestamp) -> Result<(), InvestingPodValidationError> {
        use InvestingPodValidationError::*;

        if self.funding_date < now {
            return Err(FundingDateMustBeInFuture);
        }

        if self.medias.is_empty() {
            return Err(PodMustHaveAtLeastOneMedia);
        }

        if self.spread > 1 {
            return Err(SpreadMustBeSmallerThanOne);
        }

        if self.funding_token_price.is_zero() {
            return Err(FundingTokenPriceCannotBeZero);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout))]
pub struct InvestingPodState {
    pub status: InvestingPodStatus,
    pub registered_media: u32,
    pub total_media: u32,
    pub supply_released: Balance,
    pub raised_funds: Balance,
}

impl InvestingPodState {
    pub fn increment_registered_media(&mut self) {
        self.registered_media += 1;
        if self.registered_media == self.total_media {
            self.status = InvestingPodStatus::Investing
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout))]
pub enum InvestingPodStatus {
    Formation,
    Investing,
    Trading,
}

impl InvestingPodStatus {
    pub fn is_investing(&self) -> bool { matches!(self, InvestingPodStatus::Investing) }

    pub fn is_trading(&self) -> bool { matches!(self, InvestingPodStatus::Trading) }

    pub fn is_formation(&self) -> bool { matches!(self, InvestingPodStatus::Formation) }
}
