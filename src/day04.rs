use std::fmt::Display;

use crate::Answer;
use color_eyre::eyre::{Result, eyre};
use log::debug;
use winnow::{
    ascii::newline,
    combinator::{repeat, separated},
    prelude::*,
    token::one_of,
};

/// Runs Day 04 - Printing Department
pub fn run(input: &str) -> Result<Answer<usize>> {
    let grid = parse(input)?;

    let part_1 = grid.accessible_rolls();

    Ok(Answer {
        part_1,
        part_2: None,
    })
}

/// A grid of rolls of paper and empty positions
#[derive(Debug, Clone, PartialEq, Eq)]
struct Grid {
    positions: Vec<Vec<Position>>,
    width: usize,
    height: usize,
}

/// Valid positions for a grid
#[derive(Debug, Clone, PartialEq, Eq)]
enum Position {
    Paper,
    Empty,
}

impl TryFrom<Vec<Vec<Position>>> for Grid {
    type Error = color_eyre::eyre::Report;

    fn try_from(value: Vec<Vec<Position>>) -> Result<Self> {
        // incredibly needlessly fancy rust way to assert all rows have the same length
        let all_rows_same_length = value
            .iter()
            .map(Vec::len)
            .collect::<Vec<_>>()
            .array_windows()
            .all(|[row1_len, row2_len]| row1_len == row2_len);

        if !all_rows_same_length {
            return Err(eyre!("All rows are not the same length"));
        }

        let width = value.first().map_or(0, Vec::len);
        let height = value.len();

        Ok(Self {
            positions: value,
            width,
            height,
        })
    }
}

impl Grid {
    /// Return the sum of all accessible rolls on the grid.
    fn accessible_rolls(&self) -> usize {
        self.positions
            .iter()
            .enumerate()
            .map(|(row_index, row)| {
                row.iter()
                    .enumerate()
                    .filter(|(column, position)| {
                        matches!(position, Position::Paper)
                            && self
                                .neighbors(*column, row_index)
                                .into_iter()
                                .filter(|(position, _, _)| matches!(position, Position::Paper))
                                .count()
                                < 4
                    })
                    .inspect(|(column, _)| debug!("({row_index}, {column}) is accessible"))
                    .count()
            })
            .sum()
    }

    /// Given a coordinate, return all neighbors.
    fn neighbors(&self, column: usize, row: usize) -> Vec<(Position, usize, usize)> {
        // There are 8 possible positions.
        // We have to filter out impossible options
        //
        // Must be enforced by the type, since usize is unsigned:
        // is x 0? if so, there are no neighbors to the left
        // is y 0? if so, there are no neighbors above
        //
        // Must be enforced by us:
        // is x self.width? if so there are no neighbors to the right
        // is y self.width? if so there are no neighbors below

        [
            (column.checked_sub(1), row.checked_sub(1)),
            (Some(column), row.checked_sub(1)),
            (Some(column + 1), row.checked_sub(1)),
            (column.checked_sub(1), Some(row)),
            (Some(column + 1), Some(row)),
            (column.checked_sub(1), Some(row + 1)),
            (Some(column), Some(row + 1)),
            (Some(column + 1), Some(row + 1)),
        ]
        .into_iter()
        // Take out neighbors with negative coordinates (too left or above)
        .filter_map(|(column, row)| Some((column?, row?)))
        // Take out neighbors that go beyond the grid coordinates (too right or below)
        .filter(|(column, row)| *column < self.width && *row < self.height)
        .map(|(column, row)| {
            (
                self.positions
                    .get(row)
                    .expect("valid index")
                    .get(column)
                    .expect("valid_index")
                    .clone(),
                column,
                row,
            )
        })
        .collect()
    }
}

/// Parse rows separated by newlines into a grid
fn parse(input: &str) -> Result<Grid> {
    separated(1.., row, newline)
        .map(|grid: Vec<Vec<Position>>| grid.try_into())
        .parse(input.trim_end())
        .map_err(|e| eyre!(e.to_string()))?
}

/// Parse a row of positions
fn row(input: &mut &str) -> winnow::Result<Vec<Position>> {
    repeat(1.., position).parse_next(input)
}

/// Parse a position character
fn position(input: &mut &str) -> winnow::Result<Position> {
    one_of(['@', '.'])
        .map(|position| match position {
            '@' => Position::Paper,
            '.' => Position::Empty,
            _ => unreachable!(),
        })
        .parse_next(input)
}

impl Display for Position {
    /// Print in the same format as test input
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Position::Paper => "@",
                Position::Empty => ".",
            }
        )
    }
}

impl Display for Grid {
    /// Print in the same format as test input
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.positions
                .iter()
                .map(|row| row.iter().map(ToString::to_string).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::enable_logging;

    const EXAMPLE_INPUT: &str = "..@@.@@@@.
@@@.@.@.@@
@@@@@.@.@@
@.@@@@..@.
@@.@@@@.@@
.@@@@@@@.@
.@.@.@.@@@
@.@@@.@@@@
.@@@@@@@@.
@.@.@@@.@.";

    #[test]
    fn part1_example() {
        enable_logging();
        assert_eq!(13, run(EXAMPLE_INPUT).expect("valid input").part_1);
    }

    #[test]
    fn parse_example() {
        let parsed = parse(EXAMPLE_INPUT).expect("valid parsing");
        // Test by parsing, then using the Display impl to print the original data
        assert_eq!(EXAMPLE_INPUT, parsed.to_string());
        assert_eq!(10, parsed.width);
        assert_eq!(10, parsed.height);
    }
}
