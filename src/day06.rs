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
    let mut problem_part1 = parse_part1(input)?;
    // Swap our numbers from columns into rows
    problem_part1.rows = transpose(&problem_part1.rows);

    let part_1 = problem_part1.evaluate();

    Ok(Answer {
        part_1,
        part_2: None,
    })
}

impl Problem {
    /// Evaluate the problem.
    fn evaluate(&self) -> Number {
        self.rows
            .iter()
            .zip(&self.operations)
            .map(|(column, operation)| {
                column
                    .iter()
                    .fold(operation.identity(), |accumulator, number| {
                        operation.op(accumulator, *number)
                    })
            })
            .sum()
    }
}

/// Exchange rows for columns.
fn transpose<T: Clone>(input: &[Vec<T>]) -> Vec<Vec<T>> {
    // Assert all rows are the same length, needed for transpose
    debug_assert!(
        input
            .iter()
            .map(Vec::len)
            .collect::<Vec<_>>()
            .array_windows()
            .all(|[len1, len2]| len1 == len2)
    );

    // This is the number of rows from our input - now the number of columns
    let new_number_of_columns = input.len();
    // This is the number of columns from our input - now the number of rows
    let new_number_of_rows = input.first().map(Vec::len).unwrap_or_default();

    let mut transposed: Vec<_> = (0..new_number_of_rows)
        .map(|_| Vec::with_capacity(new_number_of_columns))
        .collect();

    for row in input {
        // new_row is the column the number was in - now it's the row its going in
        for (new_row, number) in row.iter().enumerate() {
            transposed
                .get_mut(new_row)
                .expect("new_row <= new_number_of_rows")
                .push(number.clone());
        }
    }

    transposed
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
fn parse_part1(input: &str) -> Result<Problem> {
    problem_part1
        .parse(input.trim_end())
        .map_err(|e| eyre!(e.to_string()))
}

/// Parse a problem.
fn problem_part1(input: &mut &str) -> winnow::Result<Problem> {
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

/// Parse a single operation.
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
            parse_part1(EXAMPLE_INPUT).expect("valid input")
        );
    }
}
