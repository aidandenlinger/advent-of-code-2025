use std::str::FromStr;

use color_eyre::eyre::{Result, eyre};

use crate::Answer;
use winnow::{
    ascii::{dec_uint, newline, space0, space1},
    combinator::{preceded, separated, seq},
    prelude::*,
    token::one_of,
};

/// Runs Day 6 - Trash Compactor
pub fn run(input: &str) -> Result<Answer<Number>> {
    let problem = parse(input)?;

    let part_1 = problem.evaluate();

    Ok(Answer {
        part_1,
        part_2: None,
    })
}

impl Problem {
    /// Evaluate the problem.
    fn evaluate(&self) -> Number {
        // Assert there are rows and all rows are the same length
        debug_assert!(!self.rows.is_empty());
        debug_assert!(
            self.rows
                .iter()
                .map(Vec::len)
                .collect::<Vec<_>>()
                .array_windows()
                .all(|[len1, len2]| len1 == len2)
        );

        // Transpose into columns
        // OPTIMIZATION: instead of saving the entire column and then
        // collapsing it, we could instead only store the answer.
        // so for addition columns initalize a 0 value, for multiplication
        // columns initalize a 1 value, and then instead of pushing values
        // into a column vector we simply add/multiply the number.
        let mut columns: Vec<Vec<Number>> = (0..self.rows[0].len())
            .map(|_| Vec::with_capacity(self.rows.len()))
            .collect();

        for row in &self.rows {
            for (column_index, number) in row.iter().enumerate() {
                columns[column_index].push(*number);
            }
        }

        columns
            .into_iter()
            .zip(&self.operations)
            .map(|(column, operation)| {
                column
                    .into_iter()
                    .fold(operation.identity(), |accumulator, number| {
                        operation.op(accumulator, number)
                    })
            })
            .sum()
    }
}

/// The number type used for this homework.
type Number = u64;

/// An entire cephalopod homework problem.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Problem {
    /// All of the rows of numbers.
    rows: Vec<Vec<Number>>,
    /// The operation to be performed for each column.
    operations: Vec<Operation>,
}

/// Allowed mathematical operations.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Operation {
    Addition,
    Multiplication,
}

/// Parse input into a Problem struct.
fn parse(input: &str) -> Result<Problem> {
    problem
        .parse(input.trim_end())
        .map_err(|e| eyre!(e.to_string()))
}

/// Parse a problem.
fn problem(input: &mut &str) -> winnow::Result<Problem> {
    seq!(Problem {
        rows: separated(1.., numbers, newline),
        _: (space0, newline),
        operations: operations,
        _: space0
    })
    .parse_next(input)
}

/// Parse a row of numbers.
fn numbers(input: &mut &str) -> winnow::Result<Vec<Number>> {
    separated(1.., number, space1).parse_next(input)
}

/// Parse a single number.
fn number(input: &mut &str) -> winnow::Result<Number> {
    preceded(space0, dec_uint).parse_next(input)
}

/// Parse a row of operations.
fn operations(input: &mut &str) -> winnow::Result<Vec<Operation>> {
    separated(1.., operation, space1).parse_next(input)
}

fn operation(input: &mut &str) -> winnow::Result<Operation> {
    preceded(
        space0,
        one_of(['+', '*']).map(|operation| match operation {
            '+' => Operation::Addition,
            '*' => Operation::Multiplication,
            _ => unreachable!(),
        }),
    )
    .parse_next(input)
}

impl Operation {
    /// A safe inital value to start performing the value onto.
    fn identity(&self) -> Number {
        match self {
            Operation::Addition => 0,
            Operation::Multiplication => 1,
        }
    }

    /// Perform the operation with the given numbers.
    fn op(&self, left: Number, right: Number) -> Number {
        match self {
            Operation::Addition => left + right,
            Operation::Multiplication => left * right,
        }
    }
}

impl FromStr for Operation {
    type Err = color_eyre::eyre::Report;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "+" => Ok(Self::Addition),
            "*" => Ok(Self::Multiplication),
            _ => Err(eyre!("{s} is not a valid operation, expected '+' or '*'")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::enable_logging;

    const EXAMPLE_INPUT: &str = "123 328  51 64
 45 64  387 23
  6 98  215 314
*   +   *   +  ";

    #[test]
    fn part1_example() {
        enable_logging();
        assert_eq!(4_277_556, run(EXAMPLE_INPUT).expect("valid input").part_1);
    }

    #[test]
    fn parse_example() {
        enable_logging();
        assert_eq!(
            Problem {
                rows: vec![
                    vec![123, 328, 51, 64],
                    vec![45, 64, 387, 23],
                    vec![6, 98, 215, 314]
                ],
                operations: ["*", "+", "*", "+"]
                    .into_iter()
                    .flat_map(Operation::from_str)
                    .collect()
            },
            parse(EXAMPLE_INPUT).expect("valid input")
        );
    }
}
