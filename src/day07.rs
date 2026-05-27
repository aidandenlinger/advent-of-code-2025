use std::{fmt::Display, marker::PhantomData};

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
        part_2: Some(manifold.quantum_simulate()?.timelines),
    })
}

/// A Tachiyon manifold diagram, with `PhantomData` to mark if it's been simulated or not
#[derive(Debug, Clone, PartialEq, Eq)]
struct Manifold<State: ManifoldState>(Vec<Vec<ManifoldItem>>, PhantomData<State>);

/// Marker trait for the allowed `ManifoldState` types
trait ManifoldState {}

/// A Manifold diagram that hasn't been simulated yet
#[derive(Debug, Clone, PartialEq, Eq)]
struct NotSimulated;
/// A simulated Manifold diagram with a fully drawn beam that prevents resimulation.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Simulated;

// Mark them as allowed manifold states
impl ManifoldState for NotSimulated {}
impl ManifoldState for Simulated {}

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

/// Result of simulating a beam.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SimulationResult {
    /// The full manifold diagram after the beam has been split
    manifold: Manifold<Simulated>,
    /// The number of times the beam was split during simulation (for part 1)
    split_count: Number,
}

/// Result of simulating a beam.
#[derive(Debug, Clone, PartialEq, Eq)]
struct QuantumSimulationResult {
    /// Count of timelines the beam could've followed
    timelines: Number,
}

impl Manifold<NotSimulated> {
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
            manifold: Manifold(rows, PhantomData),
            split_count,
        })
    }

    /// Simulate all timelines for the beam
    fn quantum_simulate(&self) -> Result<QuantumSimulationResult> {
        let mut timelines = 0;
        // Reverse so we can pop off the back of the vec
        let reversed: Vec<_> = self.0.iter().rev().cloned().collect();

        let (first, rest) = reversed.split_last().ok_or_eyre("Empty manifest")?;

        // Left is the simulation we're building, right is the rows we haven't considered yet
        let mut timeline_queue = vec![(first.clone(), Vec::from(rest))];

        while let Some((last_row, mut grid_to_go)) = timeline_queue.pop() {
            let current_row = grid_to_go.pop();

            if let Some(mut current_row) = current_row {
                // Doing it this way to panic if we have multiple beams
                let beam_index = match last_row
                    .iter()
                    .enumerate()
                    .filter_map(|(index, item)| match item {
                        ManifoldItem::Beam => Some(index),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .as_slice()
                {
                    [beam_index] => *beam_index,
                    _ => unreachable!("Only one beam in quantum simulation"),
                };

                let value = current_row
                    .get(beam_index)
                    .expect("Valid index from last row");

                match value {
                    ManifoldItem::Empty => {
                        // We're just dropping the beam down, continue to next iteration
                        *current_row.get_mut(beam_index).expect("same as value") =
                            ManifoldItem::Beam;
                        timeline_queue.push((current_row, grid_to_go));
                    }
                    ManifoldItem::Splitter => {
                        if beam_index >= 1
                            && let Some(left) = current_row.get(beam_index - 1)
                            && matches!(left, ManifoldItem::Empty)
                        {
                            // We'll do cloning here since it's the first branch
                            let mut left_timeline = current_row.clone();
                            left_timeline[beam_index - 1] = ManifoldItem::Beam;
                            timeline_queue.push((left_timeline, grid_to_go.clone()));
                        }

                        if let Some(right) = current_row.get_mut(beam_index + 1)
                            && matches!(right, ManifoldItem::Empty)
                        {
                            *right = ManifoldItem::Beam;
                            timeline_queue.push((current_row, grid_to_go));
                        }
                    }
                    ManifoldItem::Beam => unreachable!("Only one beam in quantum simulation"),
                }
            } else {
                // This timeline is done if there's no more rows! Report it and bail
                timelines += 1;
            }
        }

        Ok(QuantumSimulationResult { timelines })
    }
}

/// Parse rows separated by newlines into a Tachyon Manifold
fn parse(input: &str) -> Result<Manifold<NotSimulated>> {
    separated(1.., row, newline)
        .map(|manifold| Manifold(manifold, PhantomData))
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

impl<State: ManifoldState> Display for Manifold<State> {
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
        trace!("\n{}", answer.manifold);
        assert_eq!(21, answer.split_count);
    }

    #[test]
    fn part2_example() {
        enable_logging();
        let answer = parse(EXAMPLE_INPUT)
            .expect("valid parsing")
            .quantum_simulate()
            .expect("valid simulation");
        assert_eq!(40, answer.timelines);
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
