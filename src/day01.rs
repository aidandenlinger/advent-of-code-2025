use color_eyre::eyre::Result;
use nom::character::complete::digit1;
use nom::character::complete::newline;
use nom::character::one_of;
use nom::combinator::{all_consuming, map_res};
use nom::multi::separated_list1;
use nom::sequence::pair;
use nom::{Finish, IResult, Parser};

/// Run the day 1 puzzle - Secret Entrance
pub fn run(input: &str) -> Result<i32> {
    const MAX_SAFE_VALUE: i32 = 100;
    const MAGIC_VALUE: i32 = 0;

    let rotations = parse(input)?;

    Ok(rotations
        .into_iter()
        .fold((50, 0), |(state, times_at_zero), rotation| {
            let new_state = match rotation {
                Rotation::Left { distance } => (state - distance) % MAX_SAFE_VALUE,
                Rotation::Right { distance } => (state + distance) % MAX_SAFE_VALUE,
            };

            let new_times_at_zero = if state == MAGIC_VALUE {
                times_at_zero + 1
            } else {
                times_at_zero
            };

            (new_state, new_times_at_zero)
        })
        .1)
}

/// A rotation per the instructions
#[derive(Debug, PartialEq, Eq, Clone)]
enum Rotation {
    Left { distance: i32 },
    Right { distance: i32 },
}

/// Given the input string, parse it into a list of rotations
fn parse(input: &str) -> Result<Vec<Rotation>> {
    Ok(all_consuming(separated_list1(newline, rotation))
        .parse(input.trim_end())
        .map_err(<nom::Err<nom::error::Error<&str>>>::to_owned)
        .finish()?
        .1)
}

/// Parse a single rotation
fn rotation(input: &str) -> IResult<&str, Rotation> {
    pair(one_of("LR"), i32)
        .map(|(direction, distance)| match direction {
            'L' => Rotation::Left { distance },
            'R' => Rotation::Right { distance },
            _ => unreachable!(),
        })
        .parse(input)
}

/// Parse an i32 from a string.
fn i32(input: &str) -> IResult<&str, i32> {
    map_res(digit1, str::parse).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("L68", &[Rotation::Left { distance: 68 }] ; "single rotation")]
    #[test_case("L68\n", &[Rotation::Left { distance: 68 }] ; "trailing newline single")]
    #[test_case("L68\nR48", &[Rotation::Left { distance: 68 }, Rotation::Right { distance: 48 }]; "multiple rotations")]
    #[test_case("L68\nR48\n", &[Rotation::Left { distance: 68 }, Rotation::Right { distance: 48 }] ; "trailing newline")]
    fn test_parser(input: &str, expected: &[Rotation]) {
        assert_eq!(expected, parse(input).expect("valid parsing"));
    }

    #[test]
    fn test_input_part1() {
        assert_eq!(
            3,
            run("L68
L30
R48
L5
R60
L55
L1
L99
R14
L82")
            .expect("valid input")
        );
    }
}
