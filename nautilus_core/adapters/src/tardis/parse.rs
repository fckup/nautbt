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

use nautilus_core::{datetime::NANOSECONDS_IN_MICROSECOND, nanos::UnixNanos};
use nautilus_model::{
    data::bar::BarSpecification,
    enums::{AggressorSide, BarAggregation, BookAction, OptionKind, OrderSide, PriceType},
    identifiers::{InstrumentId, Symbol},
};

use super::enums::{Exchange, OptionType};

#[must_use]
#[inline]
pub fn parse_symbol_str(symbol: &str) -> String {
    // TODO: Implement symbol standardization for Binance, Bybit and dYdX
    symbol.to_uppercase()
}

/// Parses a Nautilus instrument ID from the given Tardis `exchange` and `symbol` values.
#[must_use]
pub fn parse_instrument_id(exchange: &Exchange, symbol: &str) -> InstrumentId {
    let symbol = Symbol::from_str_unchecked(parse_symbol_str(symbol));
    InstrumentId::new(symbol, exchange.as_venue())
}

/// Parses a Nautilus order side from the given Tardis string `value`.
#[must_use]
pub fn parse_order_side(value: &str) -> OrderSide {
    match value {
        "bid" => OrderSide::Buy,
        "ask" => OrderSide::Sell,
        _ => OrderSide::NoOrderSide,
    }
}

/// Parses a Nautilus aggressor side from the given Tardis string `value`.
#[must_use]
pub fn parse_aggressor_side(value: &str) -> AggressorSide {
    match value {
        "buy" => AggressorSide::Buyer,
        "sell" => AggressorSide::Seller,
        _ => AggressorSide::NoAggressor,
    }
}

/// Parses a Nautilus option kind from the given Tardis enum `value`.
#[must_use]
pub const fn parse_option_kind(value: OptionType) -> OptionKind {
    match value {
        OptionType::Call => OptionKind::Call,
        OptionType::Put => OptionKind::Put,
    }
}

/// Parses a UNIX nanoseconds timestamp from the given Tardis microseconds `value_us`.
#[must_use]
pub fn parse_timestamp(value_us: u64) -> UnixNanos {
    UnixNanos::from(value_us * NANOSECONDS_IN_MICROSECOND)
}

/// Parses a Nautilus book action inferred from the given Tardis values.
#[must_use]
pub fn parse_book_action(is_snapshot: bool, amount: f64) -> BookAction {
    if is_snapshot {
        BookAction::Add
    } else if amount == 0.0 {
        BookAction::Delete
    } else {
        BookAction::Update
    }
}

/// Parses a Nautilus bar specification from the given Tardis string `value`.
///
/// The [`PriceType`] is always `LAST` for Tardis trade bars.
#[must_use]
pub fn parse_bar_spec(value: &str) -> BarSpecification {
    let parts: Vec<&str> = value.split('_').collect();
    let last_part = parts.last().expect("Invalid bar spec");
    let split_idx = last_part
        .chars()
        .position(|c| !c.is_ascii_digit())
        .expect("Invalid bar spec");

    let (step_str, suffix) = last_part.split_at(split_idx);
    let step: usize = step_str.parse().expect("Invalid step");

    let aggregation = match suffix {
        "ms" => BarAggregation::Millisecond,
        "s" => BarAggregation::Second,
        "m" => BarAggregation::Minute,
        "ticks" => BarAggregation::Tick,
        "vol" => BarAggregation::Volume,
        _ => panic!("Unsupported bar aggregation type"),
    };

    BarSpecification {
        step,
        aggregation,
        price_type: PriceType::Last,
    }
}

////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use nautilus_model::enums::AggressorSide;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(Exchange::OkexFutures, "BTC-USD-200313", "BTC-USD-200313.OKEX")]
    #[case(Exchange::Binance, "ETH-USDT", "ETH-USDT.BINANCE")]
    #[case(Exchange::Bitmex, "XBTUSD", "XBTUSD.BITMEX")]
    #[case(Exchange::HuobiDmLinearSwap, "FOO-BAR", "FOO-BAR.HUOBI")]
    fn test_parse_instrument_id(
        #[case] exchange: Exchange,
        #[case] symbol: &str,
        #[case] expected: &str,
    ) {
        let instrument_id = parse_instrument_id(&exchange, symbol);
        let expected_instrument_id = InstrumentId::from_str(expected).unwrap();
        assert_eq!(instrument_id, expected_instrument_id);
    }

    #[rstest]
    #[case("bid", OrderSide::Buy)]
    #[case("ask", OrderSide::Sell)]
    #[case("unknown", OrderSide::NoOrderSide)]
    #[case("", OrderSide::NoOrderSide)]
    #[case("random", OrderSide::NoOrderSide)]
    fn test_parse_order_side(#[case] input: &str, #[case] expected: OrderSide) {
        assert_eq!(parse_order_side(input), expected);
    }

    #[rstest]
    #[case("buy", AggressorSide::Buyer)]
    #[case("sell", AggressorSide::Seller)]
    #[case("unknown", AggressorSide::NoAggressor)]
    #[case("", AggressorSide::NoAggressor)]
    #[case("random", AggressorSide::NoAggressor)]
    fn test_parse_aggressor_side(#[case] input: &str, #[case] expected: AggressorSide) {
        assert_eq!(parse_aggressor_side(input), expected);
    }

    #[rstest]
    fn test_parse_timestamp() {
        let input_timestamp: u64 = 1583020803145000;
        let expected_nanos: UnixNanos =
            UnixNanos::from(input_timestamp * NANOSECONDS_IN_MICROSECOND);

        assert_eq!(parse_timestamp(input_timestamp), expected_nanos);
    }

    #[rstest]
    #[case(true, 10.0, BookAction::Add)]
    #[case(false, 0.0, BookAction::Delete)]
    #[case(false, 10.0, BookAction::Update)]
    fn test_parse_book_action(
        #[case] is_snapshot: bool,
        #[case] amount: f64,
        #[case] expected: BookAction,
    ) {
        assert_eq!(parse_book_action(is_snapshot, amount), expected);
    }

    #[rstest]
    #[case("trade_bar_10ms", 10, BarAggregation::Millisecond)]
    #[case("trade_bar_5m", 5, BarAggregation::Minute)]
    #[case("trade_bar_100ticks", 100, BarAggregation::Tick)]
    #[case("trade_bar_100000vol", 100000, BarAggregation::Volume)]
    fn test_parse_bar_spec(
        #[case] value: &str,
        #[case] expected_step: usize,
        #[case] expected_aggregation: BarAggregation,
    ) {
        let spec = parse_bar_spec(value);
        assert_eq!(spec.step, expected_step);
        assert_eq!(spec.aggregation, expected_aggregation);
        assert_eq!(spec.price_type, PriceType::Last);
    }

    #[rstest]
    #[case("trade_bar_10unknown")]
    #[should_panic(expected = "Unsupported bar aggregation type")]
    fn test_parse_bar_spec_invalid_suffix(#[case] value: &str) {
        let _ = parse_bar_spec(value);
    }

    #[rstest]
    #[case("")]
    #[should_panic(expected = "Invalid bar spec")]
    fn test_parse_bar_spec_empty(#[case] value: &str) {
        let _ = parse_bar_spec(value);
    }

    #[rstest]
    #[case("trade_bar_notanumberms")]
    #[should_panic(expected = "Invalid step")]
    fn test_parse_bar_spec_invalid_step(#[case] value: &str) {
        let _ = parse_bar_spec(value);
    }
}
