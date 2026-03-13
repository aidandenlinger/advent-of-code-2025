use std::ops::RangeInclusive;

use crate::Answer;
use color_eyre::eyre::{Result, eyre};
use winnow::{
    ascii::{dec_uint, newline},
    combinator::{separated, separated_pair, seq},
    prelude::*,
};

/// Runs Day 05 - Cafeteria
pub fn run(input: &str) -> Result<Answer<usize>> {
    let database = parse(input)?;

    // Optimization I haven't done: we can optimize the ranges.
    // ie 10-14 and 12-18 can be combined into 10-18
    // which can then be combined with 16-20 to get 10-20
    // so the 4 ranges can be optimized to 3-5 and 10-20
    // **but** the naive approach is quick enough :shrug:
    let part_1 = database
        .available_ids
        .iter()
        .filter(|id| {
            database
                .fresh_id_ranges
                .iter()
                .any(|range| range.contains(id))
        })
        .count();

    Ok(Answer {
        part_1,
        part_2: None,
    })
}

/// An ingredient id
type Id = u64;

/// A Database of ingredient ids
#[derive(Debug, Clone, PartialEq, Eq)]
struct Database {
    fresh_id_ranges: Vec<RangeInclusive<Id>>,
    available_ids: Vec<Id>,
}

/// Parse a database from a corresponding string.
fn parse(input: &str) -> Result<Database> {
    database
        .parse(input.trim_end())
        .map_err(|e| eyre!(e.to_string()))
}

/// Parse a database from a corresponding string.
fn database(input: &mut &str) -> winnow::Result<Database> {
    seq!(Database {
        fresh_id_ranges: separated(1.., range, newline),
        _: (newline, newline),
        available_ids: separated(1.., id, newline)
    })
    .parse_next(input)
}

/// Parse a range from a corresponding string.
fn range(input: &mut &str) -> winnow::Result<RangeInclusive<Id>> {
    separated_pair(id, '-', id)
        .map(|(start, end)| start..=end)
        .parse_next(input)
}

/// Parse an id from a corresponding string.
fn id(input: &mut &str) -> winnow::Result<Id> {
    dec_uint.parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::enable_logging;

    const EXAMPLE_INPUT: &str = "3-5
10-14
16-20
12-18

1
5
8
11
17
32";

    #[test]
    fn part1_example() {
        enable_logging();
        assert_eq!(3, run(EXAMPLE_INPUT).expect("valid input").part_1);
    }

    #[test]
    fn parse_example() {
        let parsed = parse(EXAMPLE_INPUT).expect("valid parsing");
        assert_eq!(
            &[3..=5, 10..=14, 16..=20, 12..=18],
            parsed.fresh_id_ranges.as_slice()
        );
        assert_eq!(&[1, 5, 8, 11, 17, 32], parsed.available_ids.as_slice());
    }
}
