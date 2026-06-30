use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

use crate::Answer;
use color_eyre::eyre::{Result, eyre};
use winnow::{
    Parser,
    ascii::{dec_uint, newline},
    combinator::{separated, seq},
};

/// Number type for the answer
type Number = usize;

pub fn run(input: &str) -> Result<Answer<Number>> {
    let points = parse(input)?;
    let connections = connections_to_make(&points, 1000);
    let roster = GroupRoster::new(&connections);
    let group_counts = roster.largest_groups(3);

    Ok(Answer {
        part_1: group_counts.into_iter().fold(1, usize::strict_mul),
        part_2: None,
    })
}

/// A point in 3D space.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Point {
    x: Number,
    y: Number,
    z: Number,
}

/// A struct holding the distance between two points.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Distance<'points> {
    /// The distance squared - we avoid square rooting since it isn't necessary here
    squared: Number,
    p1: &'points Point,
    p2: &'points Point,
}

impl<'points> Distance<'points> {
    fn new(p1: &'points Point, p2: &'points Point) -> Self {
        Self {
            squared: p1.distance_squared(p2),
            p1,
            p2,
        }
    }
}

impl Point {
    /// Calculates the distance squared between two points
    /// (Avoid square rooting since we're only comparing, we don't need the exact distance)
    fn distance_squared(&self, p2: &Point) -> Number {
        (p2.x.abs_diff(self.x)).pow(2)
            + (p2.y.abs_diff(self.y)).pow(2)
            + (p2.z.abs_diff(self.z)).pow(2)
    }
}

/// Find the `num_of_connections` closest pairs of points.
fn connections_to_make(points: &[Point], num_of_connections: usize) -> Vec<(Point, Point)> {
    // A *min-heap* that stores Reverse<Distance>: we want the **smallest** distances
    // Future optimization if needed: don't store all distances, only store num_of_connections distances.
    // going naive for now and storing all distances, if needed we can optimize
    let mut heap = BinaryHeap::new();

    for (index, p1) in points.iter().enumerate() {
        for p2 in points.iter().skip(index + 1) {
            heap.push(Reverse(Distance::new(p1, p2)));
        }
    }

    // Now our heap sorts all distances, pop out only the connections needed
    assert!(heap.len() >= num_of_connections);
    let mut shortest_connections = Vec::with_capacity(num_of_connections);

    for _ in 0..num_of_connections {
        shortest_connections.push(
            heap.pop()
                .map(|Reverse(Distance { p1, p2, .. })| (p1.clone(), p2.clone()))
                .expect("We checked heap had at least num_of_connections elements above"),
        );
    }

    shortest_connections
}

// When ordering, *only* consider distance_squared, not the points themselves
impl Ord for Distance<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.squared.cmp(&other.squared)
    }
}

impl PartialOrd for Distance<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Data structure to hold points together into groups.
struct GroupRoster(HashMap<Point, GroupId>);

/// Implementation detail for [`GroupRoster`], identifying a group.
type GroupId = u16;

impl GroupRoster {
    /// Construct the roster from a list of points to connect.
    fn new(connections: &[(Point, Point)]) -> Self {
        // Maps Point -> GroupId
        let mut map = HashMap::new();
        let mut unclaimed_group_id: GroupId = 1;

        for (p1, p2) in connections {
            for point in [p1, p2] {
                if !map.contains_key(point) {
                    map.insert(point.clone(), unclaimed_group_id);
                    unclaimed_group_id += 1;
                }
            }

            let p1_group = *map.get(p1).expect("We asserted entry above");
            let p2_group = *map.get(p2).expect("We asserted entry above");

            // Merge p2's group into p1 by changing all entries pointing to p2 to be p1
            for group_id in map.values_mut().filter(|group_id| **group_id == p2_group) {
                *group_id = p1_group;
            }
        }

        Self(map)
    }

    /// Return the size of the `number` largest groups.
    fn largest_groups(&self, number: usize) -> Vec<usize> {
        let mut counts = HashMap::new();

        for group in self.0.values() {
            let count = counts.entry(group).or_default();
            *count += 1;
        }

        let mut heap = counts.values().collect::<BinaryHeap<_>>();

        let mut answer = Vec::with_capacity(number);

        for _ in 0..number {
            // NOTE: if we don't have enough groups, the remainer are of size 1
            // because they'd be standalone points
            answer.push(*heap.pop().unwrap_or(&1));
        }

        answer
    }
}

/// Parse all newline separated points.
fn parse(input: &str) -> Result<Vec<Point>> {
    separated(1.., point, newline)
        .parse(input.trim_end())
        .map_err(|e| eyre!(e.to_string()))
}

/// Parse a single point.
fn point(input: &mut &str) -> winnow::Result<Point> {
    seq!(dec_uint, _: ',', dec_uint, _: ',', dec_uint)
        .map(|(x, y, z)| Point { x, y, z })
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::enable_logging;
    use test_case::test_case;

    const EXAMPLE_INPUT: &str = "162,817,812
57,618,57
906,360,560
592,479,940
352,342,300
466,668,158
542,29,236
431,825,988
739,650,466
52,470,668
216,146,977
819,987,18
117,168,530
805,96,715
346,949,466
970,615,88
941,993,340
862,61,35
984,92,344
425,690,689";

    // Sanity checking
    // Node: Math.pow(431-425, 2)+Math.pow(825-690, 2)+Math.pow(988-689,2)
    #[test_case(&Point { x: 431, y: 825, z: 988 }, &Point { x: 425, y: 690, z: 689 },
107_662 ; "simple case 1")]
    // Node: Math.pow(52-117, 2)+ Math.pow(470-168, 2) + Math.pow(668-530, 2)
    #[test_case(&Point { x: 52, y: 470, z: 668 }, &Point { x: 117, y: 168, z: 530 },   114_473 ; "simple case 2")]
    fn distance_squared(p1: &Point, p2: &Point, expected_distance: Number) {
        assert_eq!(expected_distance, p1.distance_squared(p2));
    }

    #[test]
    fn circuit_sizes() {
        enable_logging();
        let points = parse(EXAMPLE_INPUT).expect("valid parsing");
        let connections = connections_to_make(&points, 10);

        let roster = GroupRoster::new(&connections);
        let group_counts = roster.largest_groups(3);
        assert_eq!(&[5, 4, 2], group_counts.as_slice());
        assert_eq!(40, group_counts.into_iter().fold(1, usize::strict_mul));
    }

    #[test]
    fn connections() {
        enable_logging();
        let points = parse(EXAMPLE_INPUT).expect("valid parsing");
        // The text only refers to the first 4 sets
        let connections = connections_to_make(&points, 4);

        assert_eq!(
            &connections,
            &[
                // 1
                (
                    Point {
                        x: 162,
                        y: 817,
                        z: 812
                    },
                    Point {
                        x: 425,
                        y: 690,
                        z: 689
                    }
                ),
                // 2
                (
                    Point {
                        x: 162,
                        y: 817,
                        z: 812
                    },
                    Point {
                        x: 431,
                        y: 825,
                        z: 988
                    }
                ),
                // 3
                (
                    Point {
                        x: 906,
                        y: 360,
                        z: 560
                    },
                    Point {
                        x: 805,
                        y: 96,
                        z: 715
                    }
                ),
                // 4
                (
                    Point {
                        x: 431,
                        y: 825,
                        z: 988
                    },
                    Point {
                        x: 425,
                        y: 690,
                        z: 689
                    }
                ),
            ]
        );
    }

    #[allow(
        clippy::too_many_lines,
        reason = "unit test, rustfmt loves spreading the Point across multiple lines"
    )]
    #[test]
    fn parse_example() {
        enable_logging();
        let parsed = parse(EXAMPLE_INPUT).expect("valid parsing");

        assert_eq!(
            &parsed,
            &[
                Point {
                    x: 162,
                    y: 817,
                    z: 812
                },
                Point {
                    x: 57,
                    y: 618,
                    z: 57
                },
                Point {
                    x: 906,
                    y: 360,
                    z: 560
                },
                Point {
                    x: 592,
                    y: 479,
                    z: 940
                },
                Point {
                    x: 352,
                    y: 342,
                    z: 300
                },
                Point {
                    x: 466,
                    y: 668,
                    z: 158
                },
                Point {
                    x: 542,
                    y: 29,
                    z: 236
                },
                Point {
                    x: 431,
                    y: 825,
                    z: 988
                },
                Point {
                    x: 739,
                    y: 650,
                    z: 466
                },
                Point {
                    x: 52,
                    y: 470,
                    z: 668
                },
                Point {
                    x: 216,
                    y: 146,
                    z: 977
                },
                Point {
                    x: 819,
                    y: 987,
                    z: 18
                },
                Point {
                    x: 117,
                    y: 168,
                    z: 530
                },
                Point {
                    x: 805,
                    y: 96,
                    z: 715
                },
                Point {
                    x: 346,
                    y: 949,
                    z: 466
                },
                Point {
                    x: 970,
                    y: 615,
                    z: 88
                },
                Point {
                    x: 941,
                    y: 993,
                    z: 340
                },
                Point {
                    x: 862,
                    y: 61,
                    z: 35
                },
                Point {
                    x: 984,
                    y: 92,
                    z: 344
                },
                Point {
                    x: 425,
                    y: 690,
                    z: 689
                }
            ]
        );
    }
}
