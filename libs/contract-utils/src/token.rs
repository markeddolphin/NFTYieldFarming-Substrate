use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

/// The token standard of the contract
#[derive(Debug, Encode, Decode, SpreadLayout, PackedLayout, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout))]
pub enum TokenStandard {
    /// ERC-20
    Erc20,
    /// ERC-721
    Erc721,
    /// ERC-1155
    Erc1155,
}
