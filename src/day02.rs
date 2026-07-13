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
        .iter()
        .cloned()
        .map(|range| range.filter(|i| part1_invalid_id(*i)).sum::<u64>())
        .sum();

    let part_2 = Some(
        parsed
            .into_iter()
            .map(|range| range.filter(|i| part2_invalid_id(*i)).sum::<u64>())
            .sum(),
    );

    Ok(Answer { part_1, part_2 })
}

/// Returns if an id is invalid (that is, if it's a sequence of digits repeated twice)
fn part1_invalid_id(id: u64) -> bool {
    let num_as_string = id.to_string();

    // To be a repeated sequence, the length must be even
    if !num_as_string.len().is_multiple_of(2) {
        return false;
    }

    let (first_half, second_half) = num_as_string.split_at(num_as_string.len() / 2);
    debug_assert_eq!(first_half.len(), second_half.len());

    let is_invalid = first_half == second_half;

    if is_invalid {
        debug!("[Part 1] {id} is invalid");
    }

    is_invalid
}

/// Returns if an id is invalid (that is, if it's a sequence of digits repeated at least twice)
fn part2_invalid_id(id: u64) -> bool {
    let num_as_string = id.to_string();

    if num_as_string.len() < 2 {
        // to be a sequence repeated at least twice,
        // the string *must* be at least length two
        return false;
    }

    // consider where every other instance of the starting character is.
    // ie 212121 - look at the 2s, 2_2_2_
    // see if the substring between them is the same
    // Tricky case 11885_11885 - 1 is not a simple separator
    // determine where all occurances of the digits are, ie
    // 11___11___
    // sequential doesn't work, because 3rd index isn't 1
    // *but* gaps of 5 work, see if the patterns are equal

    let first_char = num_as_string
        .chars()
        .next()
        .expect("string is at least length 2");

    trace!("first char in string: {first_char}");

    let string_length = num_as_string.len();

    let potential_subsequence_lengths = num_as_string
        .match_indices(first_char)
        .skip(1) // skip the first occurance at index 0
        .inspect(|(index, _)| trace!("First char also occurs at index {index}"))
        .map(|(index, _first_char)| index)
        // This index is a repeat of our first character, so it marks
        // the length of the subsequence we're considering. If the entire string
        // is this subsequence repeated N times, the length of the string must be
        // N * this index. So filter out candidates that aren't modulo the string length
        .filter(|index| string_length.is_multiple_of(*index));

    // Now check if any of these candidates make a repeating subsequence
    for potential_length in potential_subsequence_lengths {
        let subsequence = num_as_string.split_at(potential_length).0;

        trace!("Checking potential subsequence '{subsequence}'");

        let is_repeating_subsequence = num_as_string
            .chars()
            .collect::<Vec<_>>()
            .chunks(potential_length)
            .collect::<Vec<_>>()
            // cheeky way to see if all elements in the array are equal
            .array_windows()
            .all(|[sequence1, sequence2]| sequence1 == sequence2);

        if is_repeating_subsequence {
            debug!("[Part 2] {id} is invalid, repeating sequence of '{subsequence}'");
            return is_repeating_subsequence;
        }
    }

    trace!("[Part 2] {id} is a valid id");
    false
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

    #[test]
    fn input_part2() {
        enable_logging();
        assert_eq!(
            Some(4_174_379_265),
            run(TEST_INPUT).expect("valid input").part_2
        );
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

    #[test_case(55)]
    #[test_case(6464)]
    #[test_case(123_123)]
    #[test_case(11)]
    #[test_case(22)]
    #[test_case(99)]
    #[test_case(10_10)]
    #[allow(clippy::large_digit_groups, reason = "demonstrates the pattern")]
    #[test_case(11885_11885)]
    #[test_case(222_222)]
    #[test_case(446_446)]
    #[test_case(3859_3859)]
    fn part1_invalid_ids(invalid_id: u64) {
        enable_logging();
        assert!(part1_invalid_id(invalid_id));
    }

    #[test_case(101)]
    fn part1_valid_ids(valid_id: u64) {
        enable_logging();
        assert!(!part1_invalid_id(valid_id));
    }

    #[test_case(1234_1234)]
    #[test_case(123_123_123)]
    #[test_case(12_12_12_12_12)]
    #[test_case(1_111_111)]
    #[test_case(11)]
    #[test_case(22)]
    #[test_case(99)]
    #[test_case(111)]
    #[test_case(999)]
    #[test_case(1010)]
    #[allow(clippy::large_digit_groups, reason = "demonstrates the pattern")]
    #[test_case(11885_11885)]
    #[test_case(222_222)]
    #[test_case(446_446)]
    #[test_case(3859_3859)]
    #[test_case(56_56_56)]
    #[test_case(824_824_824)]
    #[test_case(21_21_21_21_21)]
    fn part2_invalid_ids(invalid_id: u64) {
        enable_logging();
        assert!(part2_invalid_id(invalid_id));
    }

    #[test_case(101)]
    fn part2_valid_ids(valid_id: u64) {
        enable_logging();
        assert!(!part2_invalid_id(valid_id));
    }
}
