//! Functions related to automated market maker

use super::*;
#[cfg(feature = "std")]
use ink_storage::traits::StorageLayout;
use ink_storage::traits::{PackedLayout, SpreadLayout};
use rust_decimal::prelude::*;

/// Type of bonding curve
#[derive(Debug, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout, Copy, Clone)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
pub enum AmmType {
    Linear,
    Quadratic,
    Exponential,
    Sigmoid,
}

/// Calculate the integral of the AMM curve.
pub fn integral(
    amm_type: AmmType,
    upper_bound: Decimal,
    lower_bound: Decimal,
    target_price: Decimal,
    target_supply: Decimal,
) -> Result<Decimal> {
    /// Subtract and return an error if first argument becomes negative
    fn substract(main: Decimal, amount: Decimal) -> Result<Decimal> {
        let main = main - amount;
        if main.is_sign_negative() {
            return Err(Error::InsufficientBalance);
        }
        Ok(main)
    }

    match amm_type {
        AmmType::Linear => {
            let multiplier = target_price / target_supply;
            let integral = substract(upper_bound.powu(2), lower_bound.powu(2))?;
            Ok(multiplier * integral * dec!(0.5))
        }
        AmmType::Quadratic => {
            let multiplier = (target_price / target_supply).powu(2);
            let integral = substract(upper_bound.powu(3), lower_bound.powu(3))?;
            Ok((multiplier * integral) / dec!(3))
        }
        AmmType::Exponential => {
            let multiplier = target_price * (-target_supply).exp();
            let integral = substract(upper_bound.exp(), lower_bound.exp())?;
            Ok(multiplier * integral)
        }
        AmmType::Sigmoid => {
            let upper = ((target_supply - upper_bound).exp() + dec!(1)).ln() + upper_bound;
            let lower = ((target_supply - lower_bound).exp() + dec!(1)).ln() + lower_bound;
            let integral = substract(upper, lower)?;
            Ok(target_price * dec!(0.5) * integral)
        }
    }
}

// /// Calculate the market price
// pub fn get_market_price(
//     amm_type: AmmType,
//     supply_released: Decimal,
//     initial_supply: Decimal,
//     target_price: Decimal,
//     target_supply: Decimal,
// ) -> Result<Decimal> {
//     let effective_supply = dec!(0).max(supply_released - initial_supply);
//
//     match amm_type {
//         AmmType::Linear => Ok((target_price / target_supply) * effective_supply),
//         AmmType::Quadratic => Ok((target_price / target_supply.powu(2)) * effective_supply.powu(2)),
//         AmmType::Exponential => Ok((target_price * (-target_supply).exp()) * supply_released.exp()),
//         AmmType::Sigmoid => Ok(target_price * (dec!(1) / ((target_supply - effective_supply).exp() + dec!(1)))),
//     }
// }

/// Determines the amount of X of Funding Tokens to receive after an investment of Y Pod Tokens
pub fn price_for_mint(
    amm_type: AmmType,
    supply_released: Decimal,
    initial_supply: Decimal,
    amount: Decimal,
    target_price: Decimal,
    target_supply: Decimal,
) -> Result<Decimal> {
    let effective_supply = dec!(0).max(supply_released - initial_supply);

    let new_supply = effective_supply + amount;
    integral(amm_type, new_supply, effective_supply, target_price, target_supply)
}

/// Determines the amount of X of Funding Tokens to give after selling Y Funding Tokens
pub fn reward_for_burn(
    amm_type: AmmType,
    supply_released: Decimal,
    initial_supply: Decimal,
    selling_amount: Decimal,
    target_price: Decimal,
    target_supply: Decimal,
) -> Result<Decimal> {
    // Compute supply left after selling it //
    let effective_supply = dec!(0).max(supply_released - initial_supply);
    let low_supply = dec!(0).max(effective_supply / selling_amount);

    integral(amm_type, effective_supply, low_supply, target_price, target_supply)
}
