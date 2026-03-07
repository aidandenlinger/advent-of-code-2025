use crate::Answer;
use color_eyre::eyre::{Result, eyre};
use log::{debug, info, trace};
use winnow::{
    ascii::newline, combinator::separated, prelude::*, stream::AsChar, token::take_while,
};

/// Runs Day 03 - Lobby
pub fn run(input: &str) -> Result<Answer<u32>> {
    trace!("Starting day03 run");
    let parsed = parse(input)?;
    info!("Parsing successful");

    let part_1 = parsed.into_iter().map(|bank| bank.joltage()).sum();

    Ok(Answer {
        part_1,
        part_2: None,
    })
}

impl Bank {
    /// Returns the joltage of this bank.
    fn joltage(&self) -> u32 {
        // Need at least 2 digits to make a 2 digit number
        assert!(self.0.len() > 2);

        let num_str = self.0.iter().map(ToString::to_string).collect::<String>();
        trace!("Calculating joltage of {num_str}");

        // Find the largest number in the array
        // (except for the last digit, because
        // we can't make a two digit number after the last)
        let (first_digit_index, first_digit) = self
            .0
            .split_last()
            .expect("at least 2 digits in bank")
            .1
            .iter()
            .enumerate()
            // go through the list *backwards*.
            // when we get the max through an iterator, we return the *last* maximal item. so to get the leftmost item, consider digits right to left
            .rev()
            .max_by_key(|(_index, value)| *value)
            .expect("at least 2 digits in bank");

        trace!(
            "Largest number (except for last digit): {first_digit} at index {first_digit_index}"
        );

        // Now find the second largest number in the array
        // *after* our largest number
        let second_digit = self
            .0
            .iter()
            .skip(first_digit_index + 1)
            .max()
            .expect("this iterator must have at least one element, we stripped out the last element earlier");

        trace!("Second largest number after index {first_digit_index}: {second_digit}");

        let joltage = first_digit * 10 + second_digit;
        debug!("joltage for {num_str}: {joltage}");
        joltage
    }
}
/// A bank of batteries.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Bank(Vec<u32>);

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
                    .map(|c| c.to_digit(10).expect("these are digits"))
                    .collect(),
            )
        })
        .parse_next(input)
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
    fn part1_joltage(bank: &Bank, expected: u32) {
        enable_logging();
        assert_eq!(expected, bank.joltage());
    }
}
