use super::*;

/// A MultiToken with an optional TokenId
#[derive(Debug, Encode, Decode, SpreadLayout, PackedLayout, Clone, Copy)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub struct UniqueMultiToken {
    /// The MultiToken
    pub multi_token: MultiToken,
    /// Optional unique id of the token
    pub token_id: Option<TokenId>,
}

impl UniqueMultiToken {
    /// Calls `self.multi_token.tranfer` with `self.token_id`
    pub fn transfer(&mut self, to: AccountId, amount: impl Into<Option<Balance>>) -> Result<()> {
        self.multi_token.transfer(to, self.token_id, amount)
    }

    /// Calls `self.multi_token.tranfer_from` with `self.token_id`
    pub fn transfer_from(&mut self, from: AccountId, to: AccountId, amount: impl Into<Option<Balance>>) -> Result<()> {
        self.multi_token.transfer_from(from, to, self.token_id, amount)
    }

    /// Calls `self.multi_token.approve` with `self.token_id`
    pub fn approve(&mut self, spender: AccountId, amount: impl Into<Option<Balance>>) -> Result<()> {
        self.multi_token.approve(spender, self.token_id, amount)
    }

    /// Calls `self.multi_token.burn` with `self.token_id`
    pub fn burn(&mut self, amount: impl Into<Option<Balance>>) -> Result<()> {
        self.multi_token.burn(self.token_id, amount)
    }

    /// Calls `self.multi_token.burn_from` with `self.token_id`
    pub fn burn_from(&mut self, account: AccountId, amount: impl Into<Option<Balance>>) -> Result<()> {
        self.multi_token.burn_from(account, self.token_id, amount)
    }
}

impl AsRef<MultiToken> for UniqueMultiToken {
    fn as_ref(&self) -> &MultiToken { &self.multi_token }
}

impl AsMut<MultiToken> for UniqueMultiToken {
    fn as_mut(&mut self) -> &mut MultiToken { &mut self.multi_token }
}

/// Similar to `UniqueMultiToken`, but does not contain a `MultiToken`. Use for input/output.
#[derive(Debug, Encode, Decode, Clone, Copy)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct UniqueMultiTokenInfo {
    /// The `AccountId` of the deployed token contract
    pub account_id: AccountId,
    /// The standard the token adheres to
    pub standard: TokenStandard,
    /// Optional unique id of the token
    pub token_id: Option<TokenId>,
}

impl From<UniqueMultiToken> for UniqueMultiTokenInfo {
    fn from(value: UniqueMultiToken) -> Self {
        Self {
            account_id: value.multi_token.account_id,
            standard: value.multi_token.standard,
            token_id: value.token_id,
        }
    }
}

impl From<UniqueMultiTokenInfo> for UniqueMultiToken {
    fn from(value: UniqueMultiTokenInfo) -> Self {
        Self {
            multi_token: MultiToken { account_id: value.account_id, standard: value.standard },
            token_id: value.token_id,
        }
    }
}
