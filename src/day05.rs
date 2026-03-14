use std::{cmp::max, collections::VecDeque, ops::RangeInclusive};

use crate::Answer;
use color_eyre::eyre::{Result, eyre};
use winnow::{
    ascii::{dec_uint, newline},
    combinator::{separated, separated_pair, seq},
    prelude::*,
};

/// Runs Day 05 - Cafeteria
pub fn run(input: &str) -> Result<Answer<u64>> {
    let database = parse(input)?.optimize_ranges();

    let part_1 = database
        .available_ids
        .iter()
        .filter(|id| {
            database
                .fresh_id_ranges
                .iter()
                .any(|range| range.contains(id))
        })
        .count()
        .try_into()?;

    let part_2 = Some(
        database
            .fresh_id_ranges
            .into_iter()
            .map(|range| range.end() - range.start() + 1)
            .sum(),
    );

    Ok(Answer { part_1, part_2 })
}

impl Database {
    /// Condense overlapping ranges into the minimal set of ranges.
    fn optimize_ranges(mut self) -> Self {
        // Get the ranges in ascending order
        self.fresh_id_ranges.sort_unstable_by_key(|r| *r.start());

        // Now, we can optimize
        let mut ranges = self.fresh_id_ranges.into_iter();

        let fresh_id_ranges = match ranges.next() {
            None => vec![], // case where there were no ranges
            Some(first_range) => {
                ranges
                    .fold(
                        VecDeque::from([first_range]),
                        |mut optimized_ranges, current_range| {
                            let last_range = optimized_ranges
                                .pop_back()
                                .expect("this vec to always have at least one element");

                            // Imagine ranges as two horizontal lines.
                            // Since the ranges are sorted, there
                            // are two scenarios for current_range's start:
                            // - current_range's line starts within last_range,
                            //   meaning we can combine them!
                            // - current_range's line is further ahead entirely than
                            //   last_range, in which case they're independent
                            if last_range.contains(current_range.start()) {
                                // We can combine them! By definition last_range's
                                // start is <= current_range's start, we don't know which
                                // range ends longer
                                assert!(last_range.start() <= current_range.start());
                                optimized_ranges.push_back(
                                    *last_range.start()
                                        ..=*max(last_range.end(), current_range.end()),
                                );
                            } else {
                                optimized_ranges.extend([last_range, current_range]);
                            }

                            optimized_ranges
                        },
                    )
                    .into()
            }
        };

        Database {
            fresh_id_ranges,
            ..self
        }
    }
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
    fn part2_example() {
        enable_logging();
        assert_eq!(Some(14), run(EXAMPLE_INPUT).expect("valid input").part_2);
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

    #[test]
    fn optimize_ranges() {
        let optimized = parse(EXAMPLE_INPUT)
            .expect("valid parsing")
            .optimize_ranges();
        assert_eq!(&[3..=5, 10..=20], optimized.fresh_id_ranges.as_slice());
    }
}
