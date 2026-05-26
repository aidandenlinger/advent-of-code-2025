use std::fmt::Display;

use color_eyre::eyre::{OptionExt, Result, eyre};
use winnow::{
    Parser,
    ascii::newline,
    combinator::{repeat, separated},
    token::one_of,
};

use crate::Answer;

/// Number type for the answer
type Number = u64;

pub fn run(input: &str) -> Result<Answer<Number>> {
    let manifold = parse(input)?;

    Ok(Answer {
        part_1: manifold.simulate()?.split_count,
        part_2: None,
    })
}

/// Puzzle input, no beams
#[derive(Debug, Clone, PartialEq, Eq)]
struct Manifold(Vec<Vec<ManifoldItem>>);

/// Items in the Manifold
#[derive(Debug, Clone, PartialEq, Eq)]
enum ManifoldItem {
    /// A tachiyon beam. This is used for "S" when puzzle input is parsed
    Beam,
    /// Empty space
    Empty,
    /// A splitter
    Splitter,
}

/// Diagram with a fully drawn beam.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ManifoldSimulated(Vec<Vec<ManifoldItem>>);

/// Result of simulating a beam.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SimulationResult {
    /// The full manifold diagram after the beam has been split
    manifold: ManifoldSimulated,
    /// The number of times the beam was split during simulation (for part 1)
    split_count: Number,
}

impl Manifold {
    /// Draw the beam down the manifold, returning the finished diagram
    /// and number of times the beam was split.
    fn simulate(&self) -> Result<SimulationResult> {
        let (first, rest) = self.0.split_first().ok_or_eyre("Empty manifest")?;

        let inital_accumulator = (vec![first.to_owned()], 0);
        let (rows, split_count) = rest.iter().fold(
            inital_accumulator,
            |(mut rows, mut split_count), current_row| {
                let last_row = rows.last().expect("This accumulator to never be empty");
                let mut current_row_with_beams = current_row.clone();

                // From our last row, find indexes where we have a beam

                let beam_indexes =
                    last_row
                        .iter()
                        .enumerate()
                        .filter_map(|(index, item)| match item {
                            ManifoldItem::Beam => Some(index),
                            _ => None,
                        });

                // in our current row, check those indexes.
                // if it's empty, put a beam there (it's continuing down)
                // if there's a splitter, put a beam on left and right (assuming that we shouldn't overwrite splitters) and add one to the split count. ignore if left or right is out of bounds
                for beam_index in beam_indexes {
                    let value = current_row_with_beams
                        .get_mut(beam_index)
                        .expect("Valid index from last row");
                    match value {
                        ManifoldItem::Empty | ManifoldItem::Beam => *value = ManifoldItem::Beam,
                        ManifoldItem::Splitter => {
                            split_count += 1;

                            if beam_index >= 1
                                && let Some(left) = current_row_with_beams.get_mut(beam_index - 1)
                                && matches!(left, ManifoldItem::Empty)
                            {
                                *left = ManifoldItem::Beam;
                            }

                            if let Some(right) = current_row_with_beams.get_mut(beam_index + 1)
                                && matches!(right, ManifoldItem::Empty)
                            {
                                *right = ManifoldItem::Beam;
                            }
                        }
                    }
                }

                rows.push(current_row_with_beams);
                (rows, split_count)
            },
        );

        Ok(SimulationResult {
            manifold: ManifoldSimulated(rows),
            split_count,
        })
    }
}

/// Parse rows separated by newlines into a Tachyon Manifold
fn parse(input: &str) -> Result<Manifold> {
    separated(1.., row, newline)
        .map(Manifold)
        .parse(input.trim_end())
        .map_err(|e| eyre!(e.to_string()))
}

/// Parse a row from the Tachiyon Manifold
fn row(input: &mut &str) -> winnow::Result<Vec<ManifoldItem>> {
    repeat(1.., manifold_item).parse_next(input)
}

/// A single manifold item. Note that "S" is treated as a tachiyon beam.
fn manifold_item(input: &mut &str) -> winnow::Result<ManifoldItem> {
    one_of(['.', 'S', '^'])
        .map(|item| match item {
            '.' => ManifoldItem::Empty,
            'S' => ManifoldItem::Beam,
            '^' => ManifoldItem::Splitter,
            _ => unreachable!("winnow one_of enumerates our cases"),
        })
        .parse_next(input)
}

// Debug printing
impl Display for ManifoldItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ManifoldItem::Beam => "|",
                ManifoldItem::Empty => ".",
                ManifoldItem::Splitter => "^",
            }
        )
    }
}

impl Display for Manifold {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|row| row.iter().map(ToString::to_string).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

impl Display for ManifoldSimulated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Manifold::fmt(&Manifold(self.0.clone()), f)
    }
}

#[cfg(test)]
mod tests {
    use log::trace;

    use super::*;
    use crate::test_utils::enable_logging;

    const EXAMPLE_INPUT: &str = ".......S.......
...............
.......^.......
...............
......^.^......
...............
.....^.^.^.....
...............
....^.^...^....
...............
...^.^...^.^...
...............
..^...^.....^..
...............
.^.^.^.^.^...^.
...............";

    #[test]
    fn part1_example() {
        enable_logging();
        let answer = parse(EXAMPLE_INPUT)
            .expect("valid parsing")
            .simulate()
            .expect("valid simulation");
        trace!("{}", answer.manifold);
        assert_eq!(21, answer.split_count);
    }

    #[test]
    fn parse_example() {
        let parsed = parse(EXAMPLE_INPUT).expect("valid parsing");

        // Our program don't bother differentiating between "S" and
        // a tachiyon beam. So replace "S" with "|" and compare the output
        assert_eq!(EXAMPLE_INPUT.replace('S', "|"), parsed.to_string());
        // There are 16 rows
        assert_eq!(16, parsed.0.len());
        // Each row is 15 chars long
        assert!(parsed.0.iter().all(|row| row.len() == 15));
    }
}
