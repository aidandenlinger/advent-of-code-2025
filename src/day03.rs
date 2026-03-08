use std::fmt::Display;

use crate::Answer;
use color_eyre::eyre::{Result, eyre};
use log::{debug, info, trace};
use winnow::{
    ascii::newline, combinator::separated, prelude::*, stream::AsChar, token::take_while,
};

/// Runs Day 03 - Lobby
pub fn run(input: &str) -> Result<Answer<u64>> {
    trace!("Starting day03 run");
    let parsed = parse(input)?;
    info!("Parsing successful");

    let part_1 = parsed.iter().map(|bank| bank.joltage(Problem::Part1)).sum();
    let part_2 = Some(parsed.iter().map(|bank| bank.joltage(Problem::Part2)).sum());

    Ok(Answer { part_1, part_2 })
}

/// A bank of batteries.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Bank(Vec<u8>);

impl Bank {
    /// Returns the joltage of this bank.
    fn joltage(&self, batteries_to_turn_on: impl Into<usize>) -> u64 {
        let batteries_to_turn_on = batteries_to_turn_on.into();
        // Need enough batteries to turn on x batteries
        assert!(self.0.len() > batteries_to_turn_on);

        trace!("Calculating joltage of {self} when turning on {batteries_to_turn_on}");

        let mut joltage = vec![];
        let mut largest_number_index = None;

        for index_being_filled in (0..batteries_to_turn_on).rev() {
            // Consistently find the largest number in the array
            //   - after the last digit we've used
            //   - leaving enough digits so we can create a `batteries_to_turn_on` long number
            let (new_index, largest_number) = self
                .0
                .iter()
                .enumerate()
                // We want the largest number *after* the number we just used
                .skip(largest_number_index.map(|i| i + 1).unwrap_or_default())
                // go through the list *backwards*.
                // when we get the max through an iterator, we return the *last* maximal item.
                // so to get the leftmost largest number, consider digits right to left
                .rev()
                // We need to leave enough digits such that we can fill in the rest of the joltage
                .skip(index_being_filled)
                .max_by_key(|(_index, value)| *value)
                .expect("asserted at least `batteries_to_turn_on` digits in bank");

            largest_number_index = Some(new_index);

            trace!(
                "Largest number (ignoring last {index_being_filled} digits): {largest_number} at index {}",
                largest_number_index.expect("it's some by now")
            );

            joltage.push(largest_number);
        }

        let joltage = {
            let (mut result, mut power) = (0, 1);
            for digit in joltage.into_iter().rev() {
                result += u64::from(*digit) * power;
                power *= 10;
            }
            result
        };

        debug!("joltage for {self} with {batteries_to_turn_on} batteries: {joltage}");
        joltage
    }
}

/// Parse puzzle input into a series of banks
fn parse(input: &str) -> Result<Vec<Bank>> {
    separated(1.., bank, newline)
        .parse(input.trim_end())
        .map_err(|e| eyre!(e.to_string()))
}

/// Parse a string into a battery bank
fn bank(input: &mut &str) -> winnow::Result<Bank> {
    take_while(1.., AsChar::is_dec_digit)
        .map(|digit_str: &str| {
            Bank(
                digit_str
                    .chars()
                    .map(|c| {
                        c.to_digit(10)
                            .expect("these are digits")
                            .try_into()
                            .expect("these are digits")
                    })
                    .collect(),
            )
        })
        .parse_next(input)
}

impl Display for Bank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0.iter().map(ToString::to_string).collect::<String>()
        )
    }
}

/// Identifier for which problem we want to calculate the joltage for.
pub enum Problem {
    Part1,
    Part2,
}

/// Mapping from problem to batteries to turn on.
impl From<Problem> for usize {
    fn from(value: Problem) -> Self {
        match value {
            Problem::Part1 => 2,
            Problem::Part2 => 12,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::enable_logging;

    use super::*;
    use test_case::test_case;

    const EXAMPLE_INPUT: &str = "987654321111111
811111111111119
234234234234278
818181911112111";

    #[test]
    fn part1_example() {
        enable_logging();
        assert_eq!(357, run(EXAMPLE_INPUT).expect("valid input").part_1);
    }

    #[test]
    fn part2_example() {
        enable_logging();
        assert_eq!(
            Some(3_121_910_778_619),
            run(EXAMPLE_INPUT).expect("valid input").part_2
        );
    }

    #[test]
    fn example_input_parsed() {
        enable_logging();
        assert_eq!(
            vec![
                Bank(vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 1, 1, 1, 1, 1, 1],),
                Bank(vec![8, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 9],),
                Bank(vec![2, 3, 4, 2, 3, 4, 2, 3, 4, 2, 3, 4, 2, 7, 8],),
                Bank(vec![8, 1, 8, 1, 8, 1, 9, 1, 1, 1, 1, 2, 1, 1, 1],),
            ],
            parse(EXAMPLE_INPUT).expect("valid input")
        );
    }

    #[test_case(&bank(&mut "987654321111111").unwrap(), 98 ; "987654321111111")]
    #[test_case(&bank(&mut "811111111111119").unwrap(), 89 ; "811111111111119")]
    #[test_case(&bank(&mut "234234234234278").unwrap(), 78 ; "234234234234278")]
    #[test_case(&bank(&mut "818181911112111").unwrap(), 92 ; "818181911112111")]
    #[test_case(&bank(&mut "9891").unwrap(), 99 ; "use leftmost largest number")]
    fn part1_joltage(bank: &Bank, expected: u64) {
        enable_logging();
        assert_eq!(expected, bank.joltage(Problem::Part1));
    }

    #[test_case(&bank(&mut "987654321111111").unwrap(), 987_654_321_111 ; "987654321111111")]
    #[test_case(&bank(&mut "811111111111119").unwrap(), 811_111_111_119 ; "811111111111119")]
    #[test_case(&bank(&mut "234234234234278").unwrap(), 434_234_234_278 ; "234234234234278")]
    #[test_case(&bank(&mut "818181911112111").unwrap(), 888_911_112_111 ; "818181911112111")]
    fn part2_joltage(bank: &Bank, expected: u64) {
        enable_logging();
        assert_eq!(expected, bank.joltage(Problem::Part2));
    }
}
