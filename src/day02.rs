use std::ops::RangeInclusive;

use crate::Answer;
use color_eyre::eyre::{Result, eyre};
use log::{debug, info, trace};
use winnow::{
    ascii::dec_uint,
    combinator::{separated, separated_pair},
    prelude::*,
};

/// Run the day 2 puzzle - Gift Shop
pub fn run(input: &str) -> Result<Answer<u64>> {
    trace!("starting run");
    let parsed = parse(input)?;
    info!("parsing successful");

    let part_1 = parsed
        .into_iter()
        .map(|range| range.filter(|i| invalid_id(*i)).sum::<u64>())
        .sum();

    Ok(Answer {
        part_1,
        part_2: None,
    })
}

/// Returns if an id is invalid (that is, if it's a sequence of digits repeated twice)
fn invalid_id(id: u64) -> bool {
    let num_as_string = id.to_string();

    // To be a repeated sequence, the length must be even
    if !num_as_string.len().is_multiple_of(2) {
        return false;
    }

    let (first_half, second_half) = num_as_string.split_at(num_as_string.len() / 2);
    assert!(first_half.len() == second_half.len());

    let is_invalid = first_half == second_half;

    if is_invalid {
        debug!("{id} is invalid");
    }

    is_invalid
}

/// A range of Ids
type IdRange = RangeInclusive<u64>;

/// Parse the input string into a list of id ranges
fn parse(input: &str) -> Result<Vec<IdRange>> {
    separated(1.., id_range, ",")
        .parse(input.trim_end())
        .map_err(|e| eyre!(e.to_string()))
}

/// Parse a single id range
fn id_range(input: &mut &str) -> winnow::Result<IdRange> {
    trace!("parsing range");
    separated_pair(dec_uint, "-", dec_uint)
        .map(|(start, end)| {
            debug!("parsed: {start}, {end}");
            start..=end
        })
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use crate::test_utils::enable_logging;

    use super::*;
    use test_case::test_case;

    const TEST_INPUT: &str = "11-22,95-115,998-1012,1188511880-1188511890,222220-222224,1698522-1698528,446443-446449,38593856-38593862,565653-565659,824824821-824824827,2121212118-2121212124";

    #[test]
    fn input_part1() {
        enable_logging();
        assert_eq!(1_227_775_554, run(TEST_INPUT).expect("valid input").part_1);
    }

    #[test_case("11-22", &[(11..=22)] ; "single range")]
    #[test_case("11-22,95-115", &[(11..=22), (95..=115)] ; "multiple ranges")]
    fn parsing_unit_tests(input: &str, expected: &[IdRange]) {
        enable_logging();
        assert_eq!(expected, parse(input).expect("valid parsing"));
    }

    #[test]
    fn parse_test_input() {
        enable_logging();
        parse(TEST_INPUT).expect("valid parsing");
    }

    #[test]
    fn invalid_ids() {
        enable_logging();
        let invalid_ids = [
            55,
            6464,
            123_123,
            11,
            22,
            99,
            10_10,
            #[allow(clippy::large_digit_groups, reason = "showcases the pattern")]
            11885_11885,
            222_222,
            446_446,
            3859_3859,
        ];

        for id in invalid_ids {
            assert!(invalid_id(id));
        }
    }

    #[test]
    fn valid_ids() {
        enable_logging();
        let valid_ids = [101];

        for id in valid_ids {
            assert!(!invalid_id(id));
        }
    }
}
