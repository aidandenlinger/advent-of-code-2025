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
    let distances = points_by_smallest_distance(&points);

    let part_1 = {
        let roster = GroupRoster::new(distances.clone(), &points, Connections::Constant(1000));
        let group_counts = roster.largest_groups(3);
        group_counts.into_iter().fold(1, usize::strict_mul)
    };

    let part_2 = Some({
        let (p1, p2) =
            GroupRoster::new(distances, &points, Connections::UntilOneGroup).last_points_connected;

        p1.x * p2.x
    });

    Ok(Answer { part_1, part_2 })
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
fn points_by_smallest_distance(points: &[Point]) -> BinaryHeap<Reverse<Distance<'_>>> {
    // A *min-heap* that stores Reverse<Distance>: we want the **smallest** distances
    // We're going to hold all combinations of two elements: C(n, 2) = (n * (n-1)) / 2
    let num_of_points = points.len();
    let mut distances = Vec::with_capacity((num_of_points * (num_of_points - 1)) / 2);

    for (index, p1) in points.iter().enumerate() {
        for p2 in &points[index + 1..] {
            distances.push(Reverse(Distance::new(p1, p2)));
        }
    }

    BinaryHeap::from(distances)
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
struct GroupRoster {
    /// Map of point to the group it's in.
    roster: HashMap<Point, GroupId>,
    /// The last two points that were connected.
    last_points_connected: (Point, Point),
}

/// Implementation detail for [`GroupRoster`], identifying a group.
type GroupId = u16;

/// How many connections to make during the group roster.
#[derive(Clone, Copy)]
enum Connections {
    /// Make a constant number of connections.
    Constant(usize),
    /// Keep making connections until every point is in the same group.
    UntilOneGroup,
}

impl GroupRoster {
    /// Construct the roster from a list of points to connect.
    fn new(
        mut connections: BinaryHeap<Reverse<Distance>>,
        points: &[Point],
        mode: Connections,
    ) -> Self {
        // Maps Point -> GroupId
        let mut roster = points.iter().cloned().zip(1..).collect::<HashMap<_, _>>();
        let mut last_points_connected = None;

        let mut connect_points = |roster: &mut HashMap<_, _>, p1, p2| {
            let p1_group = *roster
                .get(p1)
                .expect("Hashmap was initalized with all points");
            let p2_group = *roster
                .get(p2)
                .expect("Hashmap was initalized with all points");

            // If they're in the same group, do nothing
            if p1_group == p2_group {
                return;
            }

            // Merge p2's group into p1 by changing all entries pointing to p2 to be p1
            for group_id in roster
                .values_mut()
                .filter(|group_id| **group_id == p2_group)
            {
                *group_id = p1_group;
            }

            last_points_connected = Some((p1, p2));
        };

        match mode {
            Connections::Constant(num_of_pops) => {
                assert!(num_of_pops <= connections.len());
                for _ in 0..num_of_pops {
                    let Reverse(Distance { p1, p2, .. }) =
                        connections.pop().expect("Asserted enough elements");

                    connect_points(&mut roster, p1, p2);
                }
            }
            Connections::UntilOneGroup => {
                loop {
                    let Reverse(Distance { p1, p2, .. }) =
                        connections.pop().expect("Asserted enough elements");

                    connect_points(&mut roster, p1, p2);

                    // It's so annoying checking if all array elements are the same in Rust
                    let first_group_id = *roster.values().next().expect("hashmap is never empty");

                    if roster.values().all(|group_id| group_id == &first_group_id) {
                        break;
                    }
                }
            }
        }

        // a clone/unwrap
        let last_points_connected = last_points_connected
            .map(|(p1, p2)| (p1.clone(), p2.clone()))
            .expect("at least one connection");

        Self {
            roster,
            last_points_connected,
        }
    }

    /// Return the size of the `number` largest groups.
    fn largest_groups(&self, number: usize) -> Vec<usize> {
        let mut counts = HashMap::new();

        for group in self.roster.values() {
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
    fn last_points() {
        const EXPECTED_POINT_1: Point = Point {
            x: 216,
            y: 146,
            z: 977,
        };
        const EXPECTED_POINT_2: Point = Point {
            x: 117,
            y: 168,
            z: 530,
        };

        enable_logging();
        let points = parse(EXAMPLE_INPUT).expect("valid parsing");
        let connections = points_by_smallest_distance(&points);

        let roster = GroupRoster::new(connections, &points, Connections::UntilOneGroup);
        let last_points = roster.last_points_connected;

        match last_points.0 {
            EXPECTED_POINT_1 => assert_eq!(
                last_points.1, EXPECTED_POINT_2,
                "{EXPECTED_POINT_1:?} matched but {:?} was not {EXPECTED_POINT_2:?}",
                last_points.1
            ),
            EXPECTED_POINT_2 => assert_eq!(
                last_points.1, EXPECTED_POINT_1,
                "{EXPECTED_POINT_2:?} matched but {:?} was not {EXPECTED_POINT_1:?}",
                last_points.1
            ),
            _ => panic!(
                "{:?} was not {EXPECTED_POINT_1:?} or {EXPECTED_POINT_2:?}",
                last_points.0
            ),
        }

        assert_eq!(25_272, last_points.0.x * last_points.1.x);
    }

    #[test]
    fn circuit_sizes() {
        enable_logging();
        let points = parse(EXAMPLE_INPUT).expect("valid parsing");
        let connections = points_by_smallest_distance(&points);

        let roster = GroupRoster::new(connections, &points, Connections::Constant(10));
        let group_counts = roster.largest_groups(3);
        assert_eq!(&[5, 4, 2], group_counts.as_slice());
        assert_eq!(40, group_counts.into_iter().fold(1, usize::strict_mul));
    }

    #[test]
    fn connections() {
        enable_logging();
        let points = parse(EXAMPLE_INPUT).expect("valid parsing");
        // The text only refers to the first 4 sets
        let mut connections = points_by_smallest_distance(&points);

        let mut check_point = |expected_p1, expected_p2| {
            let Reverse(Distance {
                p1: actual_p1,
                p2: actual_p2,
                ..
            }) = connections.pop().expect("enough elements");
            assert_eq!(expected_p1, actual_p1, "P1 didn't match");
            assert_eq!(expected_p2, actual_p2, "P2 didn't match");
        };

        check_point(
            &Point {
                x: 162,
                y: 817,
                z: 812,
            },
            &Point {
                x: 425,
                y: 690,
                z: 689,
            },
        );
        check_point(
            &Point {
                x: 162,
                y: 817,
                z: 812,
            },
            &Point {
                x: 431,
                y: 825,
                z: 988,
            },
        );
        check_point(
            &Point {
                x: 906,
                y: 360,
                z: 560,
            },
            &Point {
                x: 805,
                y: 96,
                z: 715,
            },
        );
        check_point(
            &Point {
                x: 431,
                y: 825,
                z: 988,
            },
            &Point {
                x: 425,
                y: 690,
                z: 689,
            },
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
