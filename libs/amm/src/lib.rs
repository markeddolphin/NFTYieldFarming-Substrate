#![cfg_attr(not(feature = "std"), no_std)]

use contract_utils::env_exports::Balance;

/// Mathematical curve determining the Amm functions.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    scale::Encode,
    scale::Decode,
    ink_storage::traits::SpreadLayout,
    ink_storage::traits::PackedLayout,
)]
#[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout))]
pub enum Curve {
    Quadratic,
    Linear,
}

/// Automated market maker functionality. The Amm does not store the liquidity pool state, it just
/// governs the pricing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Amm {
    Quadratic { scale: Balance, shift: Balance },
    Linear { scale: Balance, shift: Balance },
}

/// The parameters describing an Amm. Useful to avoid incorrect usage by destructuring when doing
/// manual calculations outside of the Amm library.
///
/// ```
/// use amm::{Parameters, Amm, Curve};
/// use rust_decimal_macros::dec;
/// use contract_utils::env_exports::Balance;
///
/// let amm = Amm::new(Curve::Linear, 1_000_000_000_000, 1_000_000_000_000, 1_000_000_000_000).expect("with correct parameters, Amm::new cannot fail");
/// // now use scale and shift without being able to confuse them.
/// let Parameters { scale, shift } = amm.parameters();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parameters {
    pub scale: Balance,
    pub shift: Balance,
}

pub(crate) const THREE: u32 = 3;
pub(crate) const TWO: u32 = 2;
const BASE: u128 = 10;
const MAX_PRECISION: u32 = 12;
// Positions to truncate during integral calculation to avoid overflow
const TRUNCATE_POSITION: u32 = 6;

impl Amm {
    /// Create a new Amm. Will return None if over- or underflows occurred.
    pub fn new(curve: Curve, initial_price: Balance, max_price: Balance, max_supply: Balance) -> Option<Amm> {
        let shift = initial_price;

        let amm = match curve {
            Curve::Linear => {
                let quot = ((max_price.checked_sub(initial_price)?).checked_div_euclid(max_supply)?)
                    .checked_mul(BASE.checked_pow(MAX_PRECISION)?)?;
                let rem = ((max_price.checked_sub(initial_price)?).checked_rem_euclid(max_supply)?)
                    .checked_div(max_supply.checked_div(BASE.checked_pow(MAX_PRECISION)?)?)?;
                Amm::Linear { scale: quot.checked_add(rem)?, shift }
            }
            Curve::Quadratic => {
                let quot = (max_price.checked_sub(initial_price)?)
                    .checked_div_euclid(max_supply.checked_pow(2)?)?
                    .checked_mul(BASE.checked_pow(MAX_PRECISION)?)?;
                let rem = (max_price.checked_sub(initial_price)?)
                    .checked_rem_euclid(max_supply.checked_pow(2)?)?
                    .checked_div((max_supply.checked_div(BASE.checked_pow(MAX_PRECISION)?)?).checked_pow(2)?)?;
                Amm::Quadratic { scale: quot.checked_add(rem)?, shift }
            }
        };
        Some(amm)
    }

    /// The parameters of the Amm.
    pub fn parameters(&self) -> Parameters {
        match self {
            Amm::Quadratic { shift, scale } => Parameters { shift: *shift, scale: *scale },
            Amm::Linear { shift, scale } => Parameters { shift: *shift, scale: *scale },
        }
    }

    fn exponent(&self) -> u32 {
        match self {
            Amm::Quadratic { .. } => THREE,
            Amm::Linear { .. } => TWO,
        }
    }

    /// Computes the integral of the Amm curve
    pub fn integral(&self, lower: Balance, upper: Balance) -> Option<Balance> {
        let Parameters { shift, scale } = self.parameters();

        // truncate lower digits to avoid overflow
        let _upper = upper.checked_div(BASE.checked_pow(TRUNCATE_POSITION)?)?;
        let _lower = lower.checked_div(BASE.checked_pow(TRUNCATE_POSITION)?)?;
        let rem_pos = MAX_PRECISION.checked_sub(TRUNCATE_POSITION)?;

        let exp = self.exponent();
        let mut term1 = _upper.checked_pow(exp)?.checked_sub(_lower.checked_pow(exp)?)?;

        let mut rem_pos = rem_pos.checked_mul(exp)?;

        if rem_pos > MAX_PRECISION {
            rem_pos = rem_pos.checked_sub(MAX_PRECISION)?;
            term1 = term1.checked_div_euclid(BASE.checked_pow(rem_pos)?)?;
        } else {
            rem_pos = MAX_PRECISION.checked_sub(rem_pos)?;
            term1 = term1.checked_mul(BASE.checked_pow(rem_pos)?)?;
        }

        let term2 = upper.checked_sub(lower)?;
        let integral = term1.checked_div(exp as u128)?.checked_add(term2)?;
        scale
            .checked_mul(integral.checked_div(exp as u128)?)?
            .checked_div_euclid(BASE.checked_pow(MAX_PRECISION)?)?
            .checked_add(shift)
    }

    /// Computes the market price of the token.
    pub fn market_price(&self, supply_released: Balance) -> Option<Balance> {
        match self {
            Amm::Linear { scale, shift } => {
                scale.checked_mul(supply_released)?.checked_div(BASE.checked_pow(MAX_PRECISION)?)?.checked_add(*shift)
            }
            Amm::Quadratic { scale, shift } => scale
                .checked_mul(supply_released.checked_pow(TWO)?)?
                .checked_div(BASE.checked_pow(MAX_PRECISION)?.checked_pow(TWO)?)?
                .checked_add(*shift),
        }
    }

    /// Determines the amount of funding tokens to pay for purchasing `amount` pod tokens.
    pub fn buy(&self, supply_released: Balance, amount: Balance) -> Option<Balance> {
        self.integral(supply_released, supply_released.checked_add(amount)?)
    }

    /// Determines the amount of Y of funding tokens to receive after selling X pod tokens.
    pub fn sell(&self, supply_released: Balance, amount: Balance) -> Option<Balance> {
        let left = supply_released.checked_sub(amount)?;
        self.integral(left, supply_released)
    }
}

/// These tests have all been made to compare the output to the HLF code. They're not meant to be
/// exhaustive, just a sanity check to see that our AMM returns the same values.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_associated_constants() {
        assert_eq!(TWO, 2);
        assert_eq!(THREE, 3);
    }

    #[test]
    fn test_new() {
        assert_eq!(
            Amm::Linear { scale: 0, shift: 1_000_000_000_000 },
            Amm::new(Curve::Linear, 1_000_000_000_000, 1_000_000_000_000, 1_000_000_000_000).unwrap()
        );
        assert_eq!(
            Amm::Linear { scale: 9_000_000_000_000 as u128, shift: 1_000_000_000_000 as u128 },
            Amm::new(Curve::Linear, 1_000_000_000_000 as u128, 10_000_000_000_000 as u128, 1_000_000_000_000 as u128)
                .unwrap()
        );
        assert_eq!(
            Amm::Linear { scale: 90_000_000_000, shift: 1_000_000_000_000 },
            Amm::new(Curve::Linear, 1_000_000_000_000, 10_000_000_000_000, 100_000_000_000_000).unwrap()
        );
        assert_eq!(
            Amm::Linear { scale: 9_000_000_000_000 as u128, shift: 1_000_000_000_000 as u128 },
            Amm::new(Curve::Linear, 1_000_000_000_000 as u128, 10_000_000_000_000 as u128, 1_000_000_000_000 as u128)
                .unwrap()
        );
        assert_eq!(
            Amm::Linear { scale: 50_000_000_000, shift: 5_000_000_000_000 },
            Amm::new(Curve::Linear, 5_000_000_000_000, 10_000_000_000_000, 100_000_000_000_000).unwrap()
        );
        assert_eq!(
            Amm::Linear { scale: 990_000_000_000, shift: 1_000_000_000_000 },
            Amm::new(Curve::Linear, 1_000_000_000_000, 100_000_000_000_000, 100_000_000_000_000).unwrap()
        );
        assert_eq!(
            Amm::Linear { scale: 999_000_000_000, shift: 1_000_000_000_000 },
            Amm::new(Curve::Linear, 1_000_000_000_000, 1000_000_000_000_000, 1000_000_000_000_000).unwrap()
        );

        assert_eq!(
            Amm::Quadratic { scale: 0, shift: 1_000_000_000_000 },
            Amm::new(Curve::Quadratic, 1_000_000_000_000, 1_000_000_000_000, 1_000_000_000_000).unwrap()
        );
        assert_eq!(
            Amm::Quadratic { scale: 9_000_000_000_000, shift: 1_000_000_000_000 },
            Amm::new(Curve::Quadratic, 1_000_000_000_000, 10_000_000_000_000, 1_000_000_000_000).unwrap()
        );

        assert_eq!(
            Amm::Quadratic { scale: 900_000_000, shift: 1_000_000_000_000 },
            Amm::new(Curve::Quadratic, 1_000_000_000_000, 10_000_000_000_000, 100_000_000_000_000).unwrap()
        );

        assert_eq!(
            Amm::Quadratic { scale: 500_000_000, shift: 5_000_000_000_000 },
            Amm::new(Curve::Quadratic, 5_000_000_000_000, 10_000_000_000_000, 100_000_000_000_000).unwrap()
        );
    }

    #[test]
    fn test_sell() {
        let amm = Amm::new(Curve::Quadratic, 5_000_000_000_000, 10_000_000_000_000, 100_000_000_000_000).unwrap();
        assert_eq!(5_057_222_222_222, amm.sell(10_000_000_000_000, 10_000_000_000_000).unwrap());
        assert_eq!(18_692_000_000_000, amm.sell(100_000_000_000_000, 9_000_000_000_000).unwrap());
        assert_eq!(16_301_328_722_222, amm.sell(100_000_000_000_000, 7_300_000_000_000).unwrap());

        // Adding more range check
        assert_eq!(20_057_222_222_222, amm.sell(100_000_000_000_000, 10_000_000_000_000).unwrap());
        assert_eq!(25_040_777_777_777, amm.sell(110_000_000_000_000, 11_000_000_000_000).unwrap());
        assert_eq!(
            1_158_568_357_996_827_273_722_222,
            amm.sell(1_000_000_000_000_000_000, 7_000_300_000_000_000).unwrap()
        );

        let amm = Amm::new(Curve::Linear, 5_000_000_000_000, 10_000_000_000_000, 100_000_000_000_000).unwrap();
        assert_eq!(6_500_000_000_000, amm.sell(10_000_000_000_000, 10_000_000_000_000).unwrap());
        assert_eq!(26_712_500_000_000, amm.sell(100_000_000_000_000, 9_000_000_000_000).unwrap());
        assert_eq!(22_766_375_000_000, amm.sell(100_000_000_000_000, 7_300_000_000_000).unwrap());

        // Adding more range check
        let amm = Amm::new(Curve::Linear, 5_000_000_000_000, 10_000_000_000_000, 1000_000_000_000_000).unwrap();
        assert_eq!(23_201_637_500_000, amm.sell(1_000_000_000_000_000, 7_300_000_000_000).unwrap());
        assert_eq!(187_451_637_500_000, amm.sell(10_000_000_000_000_000, 7_300_000_000_000).unwrap());
        assert_eq!(47_561_137_500_000, amm.sell(200_000_000_000_000, 121_300_000_000_000).unwrap());
        assert_eq!(205_725_637_500_000, amm.sell(500_000_000_000_000, 200_300_000_000_000).unwrap());
        assert_eq!(20_012_250_637_500_000, amm.sell(5000_000_000_000_000, 2_000_300_000_000_000).unwrap());
        assert_eq!(249_503_790_013_087_499_324, amm.sell(5_000_000_000_000_000_000, 20_000_300_000_030_000).unwrap());
    }

    #[test]
    fn test_buy() {
        let amm = Amm::new(Curve::Quadratic, 5_000_000_000_000, 10_000_000_000_000, 100_000_000_000_000).unwrap();
        assert_eq!(5_390_555_555_555, amm.buy(10_000_000_000_000, 10_000_000_000_000).unwrap());
        assert_eq!(21_392_000_000_000, amm.buy(100_000_000_000_000, 9_000_000_000_000).unwrap());
        assert_eq!(18_077_662_055_555, amm.buy(100_000_000_000_000, 7_300_000_000_000).unwrap());

        let amm = Amm::new(Curve::Linear, 5_000_000_000_000, 10_000_000_000_000, 100_000_000_000_000).unwrap();
        assert_eq!(9_000_000_000_000, amm.buy(10_000_000_000_000, 10_000_000_000_000).unwrap());
        assert_eq!(28_737_500_000_000, amm.buy(100_000_000_000_000, 9_000_000_000_000).unwrap());
        assert_eq!(8_638_625_000_000, amm.buy(10_000_000_000_000, 9_300_000_000_000).unwrap());
        assert_eq!(248_578_625_000_000, amm.buy(100_000_000_000_000, 71_300_000_000_000).unwrap());
    }

    #[test]
    fn test_integral_linear() {
        let amm = Amm::Linear { scale: 10_000_000_000_000, shift: 1_000_000_000_000 };
        assert_eq!(1_000_000_000_000, amm.integral(0, 0).unwrap());
        assert_eq!(8_500_000_000_000, amm.integral(0, 1_000_000_000_000).unwrap());
        assert_eq!(140_437_201_000_000_000, amm.integral(1_000_000_000_000, 236_020_000_000_000).unwrap());
        assert_eq!(93511_000_000_000_000, amm.integral(136_020_000_000_000, 236_020_000_000_000).unwrap());
    }

    #[test]
    fn test_integral_quadratic() {
        let amm = Amm::Quadratic { scale: 10_000_000_000_000, shift: 1_000_000_000_000 };
        assert_eq!(1_000_000_000_000, amm.integral(0, 0).unwrap());
        // assert_eq!(5_444_444_444_444, amm.integral(0, 1_000_000_000_000).unwrap()); // original test case
        assert_eq!(5_444_444_444_440, amm.integral(0, 1_000_000_000_000).unwrap()); // slight change in satoshi
        assert_eq!(14_609_225_559_120_000_000, amm.integral(1_000_000_000_000, 236_020_000_000_000).unwrap());
        assert_eq!(11_812_592_244_444_444_440, amm.integral(136_020_000_000_000, 236_020_000_000_000).unwrap()); // slight change in satoshi
        assert_eq!(11_812_592_244_444_444_440, amm.integral(136_020_000_000_000, 236_020_000_000_000).unwrap());
        // slight change in satoshi
    }

    #[test]
    fn test_market_price_linear() {
        #[derive(Debug)]
        struct TestCase {
            initial_price: Balance,
            max_price: Balance,
            max_supply: Balance,
            supply_released: Balance,
            linear_result: Option<Balance>,
            quadratic_result: Option<Balance>,
        }

        let test_cases: Vec<TestCase> = vec![
            TestCase {
                initial_price: 1_000_000_000_000,
                max_price: 1_000_000_000_000,
                max_supply: 1_000_000_000_000,
                supply_released: 1_000_000_000_000,
                linear_result: Some(1_000_000_000_000),
                quadratic_result: Some(1_000_000_000_000),
            },
            TestCase {
                initial_price: 1_000_000_000_000,
                max_price: 100_000_000_000_000,
                max_supply: 100_000_000_000_000,
                supply_released: 1_000_000_000_000,
                linear_result: Some(1_990_000_000_000),
                quadratic_result: Some(1_009_900_000_000),
            },
            TestCase {
                initial_price: 1_000_000_000_000,
                max_price: 100_000_000_000_000,
                max_supply: 100_000_000_000_000,
                supply_released: 10_000_000_000_000,
                linear_result: Some(10_900_000_000_000),
                quadratic_result: Some(1_990_000_000_000),
            },
        ];

        for test in test_cases {
            let amm = Amm::new(Curve::Linear, test.initial_price, test.max_price, test.max_supply).unwrap();
            assert_eq!(test.linear_result, amm.market_price(test.supply_released), "linear testcase: {:?}", test);

            let amm = Amm::new(Curve::Quadratic, test.initial_price, test.max_price, test.max_supply).unwrap();
            assert_eq!(test.quadratic_result, amm.market_price(test.supply_released), "quadratic testcase: {:?}", test)
        }
    }
}
