use ink_env::{call::FromAccountId, AccountId};
use ink_prelude::{collections::BTreeMap, string::String};
#[cfg(feature = "std")]
use ink_storage::traits::StorageLayout;
use ink_storage::traits::{PackedLayout, SpreadLayout};

use erc20::Erc20;


pub type Balance = <ink_env::DefaultEnvironment as ink_env::Environment>::Balance;

#[derive(Debug, Default, Clone, PartialEq, Eq, scale::Encode, scale::Decode, PackedLayout, SpreadLayout)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct Oracle {
    pub address: AccountId,
    pub name: String,
    pub state: OracleState,
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode, PackedLayout, SpreadLayout)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum OracleState {
    Allowed,
    Disallowed,
}

impl OracleState {
    pub fn is_allowed(&self) -> bool { matches!(self, OracleState::Allowed) }

    pub fn is_disallowed(&self) -> bool { matches!(self, OracleState::Disallowed) }
}

impl Default for OracleState {
    fn default() -> Self { OracleState::Disallowed }
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct RegisterOracleRequest {
    pub address: AccountId,
    pub name: String,
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct UpdateOracleRequest {
    pub address: AccountId,
    pub name: String,
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct UpdateOracleStateRequest {
    pub address: AccountId,
    pub state: OracleState,
}

pub type Ticker = String;

#[derive(Debug, Default, PartialEq, Clone, Eq, scale::Encode, scale::Decode, PackedLayout, SpreadLayout)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct PriceBucket {
    pub token: Ticker,
    pub prices: BTreeMap<AccountId, u64>,
    pub volumes: BTreeMap<AccountId, u64>,
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct SubmitPriceRequest {
    pub token: Ticker,
    pub price: u64,
    pub volume: u64,
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct ConvertRequest {
    pub address: AccountId,
    pub amount: Balance,
}

#[derive(Debug, Clone, scale::Encode, scale::Decode, PackedLayout, SpreadLayout)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct TokenSpec {
    pub erc20: Erc20,
    pub decimal_count: u8,
    pub ticker: Ticker,
}

impl TokenSpec {
    pub fn new(erc20: Erc20) -> Result<TokenSpec, &'static str> {
        let decimal_count = erc20.decimal_count().ok_or("missing decimal_count")?;
        let ticker = erc20.symbol().ok_or("missing ticker")?;
        Ok(TokenSpec { erc20, decimal_count, ticker })
    }

    pub fn from_data(data: TokenData) -> Self {
        Self {
            decimal_count: data.decimal_count,
            ticker: data.ticker,
            erc20: FromAccountId::from_account_id(data.account_id),
        }
    }
}

#[derive(Debug, Clone, scale::Encode, scale::Decode, PackedLayout, SpreadLayout)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct TokenData {
    pub decimal_count: u8,
    pub ticker: Ticker,
    pub account_id: AccountId,
}
