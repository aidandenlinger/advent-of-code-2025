use color_eyre::eyre::{Result, eyre};
use winnow::{
    ascii::{dec_int, newline},
    combinator::{separated, seq},
    prelude::*,
    token::one_of,
};

use crate::Answer;

const MAX_SAFE_VALUE: i32 = 100;
const MAGIC_VALUE: i32 = 0;

/// Run the day 1 puzzle - Secret Entrance
pub fn run(input: &str) -> Result<Answer<i32>> {
    let rotations = parse(input)?;

    let part_1 = part_1(&rotations);
    let part_2 = Some(part_2(&rotations));

    Ok(Answer { part_1, part_2 })
}

/// Calculate Part 1 - times it points at 0
fn part_1(rotations: &[Rotation]) -> i32 {
    rotations
        .iter()
        .fold((50, 0), |(dial_pointing_at, times_at_zero), rotation| {
            // Once we add the distance, we may be over 100 or under 0.
            // To account for this, we can modulo by the MAX_SAFE_VALUE
            // which cuts out the excess.
            let new_dial_pointing_at = match rotation {
                Rotation::Left { distance } => {
                    (dial_pointing_at - distance).rem_euclid(MAX_SAFE_VALUE)
                }
                Rotation::Right { distance } => {
                    (dial_pointing_at + distance).rem_euclid(MAX_SAFE_VALUE)
                }
            };

            let new_times_at_zero = if dial_pointing_at == MAGIC_VALUE {
                times_at_zero + 1
            } else {
                times_at_zero
            };

            (new_dial_pointing_at, new_times_at_zero)
        })
        .1
}

/// Calculate Part 2 - times it passes 0
fn part_2(rotations: &[Rotation]) -> i32 {
    rotations
        .iter()
        .fold(
            (50, 0),
            |(dial_pointing_at, times_crossing_zero), rotation| {
                let (new_dial_pointing_at, distance, distance_needed_to_get_to_zero) =
                    match rotation {
                        Rotation::Left { distance } => {
                            let new_dial_pointing_at =
                                (dial_pointing_at - distance).rem_euclid(MAX_SAFE_VALUE);

                            // If we're at 5, we'd need to go left 5 to hit zero
                            // **unless we're at zero, then we need a full rotation.**
                            let distance_needed_to_get_to_zero = if dial_pointing_at == 0 {
                                MAX_SAFE_VALUE
                            } else {
                                dial_pointing_at
                            };

                            (
                                new_dial_pointing_at,
                                distance,
                                distance_needed_to_get_to_zero,
                            )
                        }
                        Rotation::Right { distance } => {
                            let new_dial_pointing_at =
                                (dial_pointing_at + distance).rem_euclid(MAX_SAFE_VALUE);

                            // If we're at 5, we'd need to go right 95 to hit zero
                            // if we're at 0, we need to go right MAX_SAFE_VALUE to hit zero
                            let distance_needed_to_get_to_zero = MAX_SAFE_VALUE - dial_pointing_at;

                            (
                                new_dial_pointing_at,
                                distance,
                                distance_needed_to_get_to_zero,
                            )
                        }
                    };

                let mut new_times_crossing_zero = times_crossing_zero;
                if distance >= &distance_needed_to_get_to_zero {
                    // Add 1 because we're currently at zero. Now, the rest of it is how many rotations of MAX_SAFE_VALUE are we hitting, which is solved with division
                    new_times_crossing_zero +=
                        1 + ((distance - distance_needed_to_get_to_zero) / MAX_SAFE_VALUE);
                }

                (new_dial_pointing_at, new_times_crossing_zero)
            },
        )
        .1
}

/// A rotation per the instructions
#[derive(Debug, PartialEq, Eq, Clone)]
enum Rotation {
    Left { distance: i32 },
    Right { distance: i32 },
}

/// Given the input string, parse it into a list of rotations
fn parse(input: &str) -> Result<Vec<Rotation>> {
    separated(1.., rotation, newline)
        .parse(input.trim_end())
        .map_err(|e| eyre!(e.to_string()))
}

/// Parse a single rotation
fn rotation(input: &mut &str) -> winnow::Result<Rotation> {
    seq!(one_of(['L', 'R']), dec_int)
        .map(|(direction, distance)| match direction {
            'L' => Rotation::Left { distance },
            'R' => Rotation::Right { distance },
            _ => unreachable!(),
        })
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("L68", &[Rotation::Left { distance: 68 }] ; "single rotation")]
    #[test_case("L68\n", &[Rotation::Left { distance: 68 }] ; "trailing newline single")]
    #[test_case("L68\nR48", &[Rotation::Left { distance: 68 }, Rotation::Right { distance: 48 }]; "multiple rotations")]
    #[test_case("L68\nR48\n", &[Rotation::Left { distance: 68 }, Rotation::Right { distance: 48 }] ; "trailing newline")]
    fn parsing(input: &str, expected: &[Rotation]) {
        assert_eq!(expected, parse(input).expect("valid parsing"));
    }

    const TEST_INPUT: &str = "L68
L30
R48
L5
R60
L55
L1
L99
R14
L82";

    #[test]
    fn input_part1() {
        assert_eq!(3, run(TEST_INPUT).expect("valid input").part_1);
    }

    #[test]
    fn input_part2() {
        assert_eq!(Some(6), run(TEST_INPUT).expect("valid input").part_2);
    }

    #[test]
    fn tricky_example_from_text() {
        assert_eq!(10, part_2(&[Rotation::Right { distance: 1000 }]));
    }

    #[test_case(&[Rotation::Right { distance: 250 }, Rotation::Left { distance: 100 }], 4 ; "test going backwards but not negative")]
    #[test_case(&[Rotation::Left { distance: 68 }], 1 ; "start of test input")]
    #[test_case(&[Rotation::Right { distance: 68 }], 1 ; "inverting start of test input")]
    #[test_case(&[Rotation::Right { distance: 40 }, Rotation::Right { distance: 10 }], 1 ; "end at zero")]
    #[test_case(&[Rotation::Right { distance: 10 }, Rotation::Right { distance: 140 }], 2 ; "60 + 140 is two rotations")]
    #[test_case(&[Rotation::Right { distance: 10 }, Rotation::Left { distance: 160 }], 2 ; "60 + -160 is two rotations")]
    fn part_2_unit_cases(rotations: &[Rotation], expected: i32) {
        assert_eq!(expected, part_2(rotations));
    }
}
