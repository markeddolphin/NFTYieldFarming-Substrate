use ink_env::{DefaultEnvironment, Environment};

#[cfg(feature = "decimal")]
pub use decimal::*;

type Balance = <DefaultEnvironment as Environment>::Balance;

/// Divide the value by 100
pub const fn percent(value: u128) -> u128 { value / 100 }

/// Converts a whole number to a balance. By default there are 12 decimal points.
pub const fn balance_from_unit(value: Balance) -> Balance { value * (10_u128.pow(crate::constants::DECIMAL_COUNT)) }

/// Extensions for rust_decimal
#[cfg(feature = "decimal")]
mod decimal {
    use super::*;
    use crate::constants::*;
    use core::str::FromStr;
    use ink_prelude::string::{String, ToString};
    use ink_storage::traits::{PackedLayout, SpreadLayout};
    use rust_decimal::prelude::*;

    /// The serialized form of `Decimal`
    pub type SerializedDecimal = [u8; 16];

    /// A Decimal that is represented by a String
    #[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode, PackedLayout, SpreadLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout))]
    #[repr(transparent)]
    pub struct DecimalString(String);

    impl core::convert::TryFrom<DecimalString> for Decimal {
        type Error = rust_decimal::Error;

        fn try_from(value: DecimalString) -> Result<Self, Self::Error> { FromStr::from_str(&value.0) }
    }

    impl From<Decimal> for DecimalString {
        fn from(value: Decimal) -> Self { Self(value.to_string()) }
    }

    /// Extensions for `SerializedDecimal`
    pub trait SerializedDecimalExt {
        /// Convert to a Decimal through deserialize
        fn deserialize_into_decimal(self) -> Decimal;
    }

    impl SerializedDecimalExt for SerializedDecimal {
        fn deserialize_into_decimal(self) -> Decimal { Decimal::deserialize(self) }
    }

    /// Extenions for Decimal
    pub trait DecimalExt {
        // fn from_balance(value: Balance) -> Decimal;
        /// Convert a Decimal to Balance with `decimal_count` decimal places
        fn into_balance(self, decimal_count: u32) -> Balance;
        fn into_privi_balance(self) -> Balance;
        fn into_balance_percent(self) -> Balance;
    }

    impl DecimalExt for Decimal {
        fn into_balance(self, decimal_count: u32) -> Balance {
            (self * Decimal::from(10).powu(decimal_count as _)).round().to_u128().expect("overflow")
        }

        fn into_privi_balance(self) -> Balance { self.into_balance(DECIMAL_COUNT as _) }

        fn into_balance_percent(self) -> Balance { self.into_balance(2) }
    }

    /// Extenions for Balance
    pub trait BalanceExt {
        /// Convert a Balance to a Decimal
        fn into_decimal(self, decimal_count: u32) -> Decimal;
        /// Convert to PRIVI decimal
        fn into_privi_decimal(self) -> Decimal;
        /// Convert to Balance with 2 decimal places
        fn into_decimal_percent(self) -> Decimal;
    }

    impl BalanceExt for Balance {
        fn into_decimal(self, decimal_count: u32) -> Decimal { Decimal::from_i128_with_scale(self as _, decimal_count) }

        fn into_privi_decimal(self) -> Decimal { self.into_decimal(DECIMAL_COUNT as _) }

        fn into_decimal_percent(self) -> Decimal { self.into_decimal(2) }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_to_from_decimal() {
            let one = Decimal::new(1, 0);

            // make sure conversion is lossless
            let balance = 88959823459713498;
            assert_eq!(balance, balance.into_privi_decimal().into_privi_balance());

            // privi conversion
            let balance = 1_000_000_000_000_u128;
            assert_eq!(balance.into_privi_decimal(), one);
            assert_eq!(one.into_privi_balance(), balance);

            // percent
            assert_eq!(100_u128.into_decimal_percent(), one);
            assert_eq!(Decimal::new(418, 2).into_balance_percent(), 418);

            // unit
            let balance = balance_from_unit(100);
            assert_eq!(balance.into_privi_decimal(), Decimal::new(100, 0));
        }
    }
}
