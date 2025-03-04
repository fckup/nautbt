// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2024 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! Represents an amount of money in a specified currency denomination.

use std::{
    cmp::Ordering,
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
    ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
    str::FromStr,
};

use nautilus_core::correctness::{check_in_range_inclusive_f64, FAILED};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};
use thousands::Separable;

use super::fixed::FIXED_PRECISION;
use crate::types::{
    currency::Currency,
    fixed::{f64_to_fixed_i64, fixed_i64_to_f64},
};

/// The maximum valid money amount which can be represented.
pub const MONEY_MAX: f64 = 9_223_372_036.0;

/// The minimum valid money amount which can be represented.
pub const MONEY_MIN: f64 = -9_223_372_036.0;

/// Represents an amount of money in a specified currency denomination.
///
/// - `MONEY_MAX` = 9_223_372_036
/// - `MONEY_MIN` = -9_223_372_036
#[repr(C)]
#[derive(Clone, Copy, Eq)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(module = "nautilus_trader.core.nautilus_pyo3.model")
)]
pub struct Money {
    /// The raw monetary amount as a signed 64-bit integer.
    /// Represents the unscaled amount, with `currency.precision` defining the number of decimal places.
    pub raw: i64,
    /// The currency denomination associated with the monetary amount.
    pub currency: Currency,
}

impl Money {
    /// Creates a new [`Money`] instance with correctness checking.
    ///
    /// # Errors
    ///
    /// This function returns an error:
    /// - If `amount` is invalid outside the representable range [-9_223_372_036, 9_223_372_036].
    /// - If `precision` is invalid outside the representable range [0, 9].
    ///
    /// # Notes
    ///
    /// PyO3 requires a `Result` type for proper error handling and stacktrace printing in Python.
    pub fn new_checked(amount: f64, currency: Currency) -> anyhow::Result<Self> {
        check_in_range_inclusive_f64(amount, MONEY_MIN, MONEY_MAX, "amount")?;

        Ok(Self {
            raw: f64_to_fixed_i64(amount, currency.precision),
            currency,
        })
    }

    /// Creates a new [`Money`] instance.
    ///
    /// # Panics
    ///
    /// This function panics:
    /// - If a correctness check fails. See [`Money::new_checked`] for more details.
    pub fn new(amount: f64, currency: Currency) -> Self {
        Self::new_checked(amount, currency).expect(FAILED)
    }

    /// Creates a new [`Money`] instance from the given `raw` fixed-point value and the specified `currency`.
    #[must_use]
    pub fn from_raw(raw: i64, currency: Currency) -> Self {
        Self { raw, currency }
    }

    /// Returns `true` if the value of this instance is zero.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.raw == 0
    }

    /// Returns the value of this instance as an `f64`.
    #[must_use]
    pub fn as_f64(&self) -> f64 {
        fixed_i64_to_f64(self.raw)
    }

    /// Returns the value of this instance as a `Decimal`.
    #[must_use]
    pub fn as_decimal(&self) -> Decimal {
        // Scale down the raw value to match the precision
        let precision = self.currency.precision;
        let rescaled_raw = self.raw / i64::pow(10, u32::from(FIXED_PRECISION - precision));
        Decimal::from_i128_with_scale(i128::from(rescaled_raw), u32::from(precision))
    }

    /// Returns a formatted string representation of this instance.
    #[must_use]
    pub fn to_formatted_string(&self) -> String {
        let amount_str = format!("{:.*}", self.currency.precision as usize, self.as_f64())
            .separate_with_underscores();
        format!("{} {}", amount_str, self.currency.code)
    }
}

impl FromStr for Money {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = value.split_whitespace().collect();

        // Ensure we have both the amount and currency
        if parts.len() != 2 {
            return Err(format!(
                "Error invalid input format '{value}'. Expected '<amount> <currency>'"
            ));
        }

        // Parse amount
        let amount = parts[0]
            .replace('_', "")
            .parse::<f64>()
            .map_err(|e| format!("Error parsing amount '{}' as `f64`: {:?}", parts[0], e))?;

        // Parse currency
        let currency = Currency::from_str(parts[1]).map_err(|e: anyhow::Error| e.to_string())?;

        Ok(Self::new(amount, currency))
    }
}

impl From<&str> for Money {
    fn from(value: &str) -> Self {
        Self::from_str(value).unwrap()
    }
}

impl From<String> for Money {
    fn from(value: String) -> Self {
        Self::from_str(value.as_str()).unwrap()
    }
}

impl From<Money> for f64 {
    fn from(money: Money) -> Self {
        money.as_f64()
    }
}

impl From<&Money> for f64 {
    fn from(money: &Money) -> Self {
        money.as_f64()
    }
}

impl Hash for Money {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
        self.currency.hash(state);
    }
}

impl PartialEq for Money {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw && self.currency == other.currency
    }
}

impl PartialOrd for Money {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }

    fn lt(&self, other: &Self) -> bool {
        assert_eq!(self.currency, other.currency);
        self.raw.lt(&other.raw)
    }

    fn le(&self, other: &Self) -> bool {
        assert_eq!(self.currency, other.currency);
        self.raw.le(&other.raw)
    }

    fn gt(&self, other: &Self) -> bool {
        assert_eq!(self.currency, other.currency);
        self.raw.gt(&other.raw)
    }

    fn ge(&self, other: &Self) -> bool {
        assert_eq!(self.currency, other.currency);
        self.raw.ge(&other.raw)
    }
}

impl Ord for Money {
    fn cmp(&self, other: &Self) -> Ordering {
        assert_eq!(self.currency, other.currency);
        self.raw.cmp(&other.raw)
    }
}

impl Neg for Money {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            raw: -self.raw,
            currency: self.currency,
        }
    }
}

impl Add for Money {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(
            self.currency, rhs.currency,
            "Currency mismatch: cannot add {} to {}",
            rhs.currency.code, self.currency.code
        );
        Self {
            raw: self
                .raw
                .checked_add(rhs.raw)
                .expect("Overflow occurred when adding `Money`"),
            currency: self.currency,
        }
    }
}

impl Sub for Money {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        assert_eq!(
            self.currency, rhs.currency,
            "Currency mismatch: cannot subtract {} from {}",
            rhs.currency.code, self.currency.code
        );
        Self {
            raw: self
                .raw
                .checked_sub(rhs.raw)
                .expect("Underflow occurred when subtracting `Money`"),
            currency: self.currency,
        }
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, other: Self) {
        assert_eq!(
            self.currency, other.currency,
            "Currency mismatch: cannot add {} to {}",
            other.currency.code, self.currency.code
        );
        self.raw = self
            .raw
            .checked_add(other.raw)
            .expect("Overflow occurred when adding `Money`");
    }
}

impl SubAssign for Money {
    fn sub_assign(&mut self, other: Self) {
        assert_eq!(
            self.currency, other.currency,
            "Currency mismatch: cannot subtract {} from {}",
            other.currency.code, self.currency.code
        );
        self.raw = self
            .raw
            .checked_sub(other.raw)
            .expect("Underflow occurred when subtracting `Money`");
    }
}

impl Add<f64> for Money {
    type Output = f64;
    fn add(self, rhs: f64) -> Self::Output {
        self.as_f64() + rhs
    }
}

impl Sub<f64> for Money {
    type Output = f64;
    fn sub(self, rhs: f64) -> Self::Output {
        self.as_f64() - rhs
    }
}

impl Mul<f64> for Money {
    type Output = f64;
    fn mul(self, rhs: f64) -> Self::Output {
        self.as_f64() * rhs
    }
}

impl Div<f64> for Money {
    type Output = f64;
    fn div(self, rhs: f64) -> Self::Output {
        self.as_f64() / rhs
    }
}

impl Debug for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({:.*}, {})",
            stringify!(Money),
            self.currency.precision as usize,
            self.as_f64(),
            self.currency,
        )
    }
}

impl Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:.*} {}",
            self.currency.precision as usize,
            self.as_f64(),
            self.currency
        )
    }
}

impl Serialize for Money {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Money {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let money_str: String = Deserialize::deserialize(deserializer)?;
        Ok(Money::from(money_str.as_str()))
    }
}

////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use float_cmp::approx_eq;
    use rstest::rstest;
    use rust_decimal_macros::dec;

    use super::*;

    #[rstest]
    fn test_debug() {
        let money = Money::new(1010.12, Currency::USD());
        let result = format!("{:?}", money);
        let expected = "Money(1010.12, USD)";
        assert_eq!(result, expected);
    }

    #[rstest]
    fn test_display() {
        let money = Money::new(1010.12, Currency::USD());
        let result = format!("{money}");
        let expected = "1010.12 USD";
        assert_eq!(result, expected);
    }

    #[rstest]
    #[should_panic]
    fn test_money_different_currency_addition() {
        let usd = Money::new(1000.0, Currency::USD());
        let btc = Money::new(1.0, Currency::BTC());
        let _result = usd + btc; // This should panic since currencies are different
    }

    #[rstest]
    fn test_money_min_max_values() {
        let min_money = Money::new(MONEY_MIN, Currency::USD());
        let max_money = Money::new(MONEY_MAX, Currency::USD());
        assert_eq!(
            min_money.raw,
            f64_to_fixed_i64(MONEY_MIN, Currency::USD().precision)
        );
        assert_eq!(
            max_money.raw,
            f64_to_fixed_i64(MONEY_MAX, Currency::USD().precision)
        );
    }

    #[rstest]
    fn test_money_addition_f64() {
        let money = Money::new(1000.0, Currency::USD());
        let result = money + 500.0;
        assert_eq!(result, 1500.0);
    }

    #[rstest]
    fn test_money_negation() {
        let money = Money::new(100.0, Currency::USD());
        let result = -money;
        assert_eq!(result.as_f64(), -100.0);
        assert_eq!(result.currency, Currency::USD().clone());
    }

    #[rstest]
    fn test_money_new_usd() {
        let money = Money::new(1000.0, Currency::USD());
        assert_eq!(money.currency.code.as_str(), "USD");
        assert_eq!(money.currency.precision, 2);
        assert_eq!(money.to_string(), "1000.00 USD");
        assert_eq!(money.to_formatted_string(), "1_000.00 USD");
        assert_eq!(money.as_decimal(), dec!(1000.00));
        assert!(approx_eq!(f64, money.as_f64(), 1000.0, epsilon = 0.001));
    }

    #[rstest]
    fn test_money_new_btc() {
        let money = Money::new(10.3, Currency::BTC());
        assert_eq!(money.currency.code.as_str(), "BTC");
        assert_eq!(money.currency.precision, 8);
        assert_eq!(money.to_string(), "10.30000000 BTC");
        assert_eq!(money.to_formatted_string(), "10.30000000 BTC");
    }

    #[rstest]
    fn test_money_serialization_deserialization() {
        let money = Money::new(123.45, Currency::USD());
        let serialized = serde_json::to_string(&money);
        let deserialized: Money = serde_json::from_str(&serialized.unwrap()).unwrap();
        assert_eq!(money, deserialized);
    }

    #[rstest]
    #[case("0USD")] // <-- No whitespace separator
    #[case("0x00 USD")] // <-- Invalid float
    #[case("0 US")] // <-- Invalid currency
    #[case("0 USD USD")] // <-- Too many parts
    #[should_panic]
    fn test_from_str_invalid_input(#[case] input: &str) {
        let _ = Money::from(input);
    }

    #[rstest]
    #[case("0 USD", Currency::USD(), dec!(0.00))]
    #[case("1.1 AUD", Currency::AUD(), dec!(1.10))]
    #[case("1.12345678 BTC", Currency::BTC(), dec!(1.12345678))]
    #[case("10_000.10 USD", Currency::USD(), dec!(10000.10))]
    fn test_from_str_valid_input(
        #[case] input: &str,
        #[case] expected_currency: Currency,
        #[case] expected_dec: Decimal,
    ) {
        let money = Money::from(input);
        assert_eq!(money.currency, expected_currency);
        assert_eq!(money.as_decimal(), expected_dec);
    }
}
