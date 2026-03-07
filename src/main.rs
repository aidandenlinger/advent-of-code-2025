//! Advent of Code driver
use std::{fmt::Display, fs::read_to_string, path::PathBuf};

use clap::{Parser, value_parser};
use color_eyre::eyre::{Context, Result};

mod day01;
mod test_utils;

#[derive(Parser)]
#[command(about)]
struct Cli {
    /// The day that should be run. Will run part 1 and part 2.
    #[arg(value_parser = value_parser!(u8).range(1..=12))]
    day: u8,

    /// Folder holding input files. "day01.txt" for day 1, "day02.txt" for
    /// day 2, etc.
    #[arg(default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/input"))]
    input_folder: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let cli = Cli::parse();

    let input_file = {
        let mut path = cli.input_folder.clone();
        path.push(format!("day{:02}.txt", cli.day));
        path
    };

    let input = read_to_string(&input_file).wrap_err(format!(
        "Couldn't read `{}` to string",
        &input_file.display()
    ))?;

    let answer = match cli.day {
        1 => day01::run(&input)?,
        _ => unimplemented!(),
    };

    println!("{answer}");

    Ok(())
}

/// Struct to contain answers for each day
pub struct Answer<T> {
    /// Answer to part 1
    part_1: T,

    /// Answer to part 2 - left as option so I can implement part 1 then do part 2
    part_2: Option<T>,
}

impl<T: Display> Display for Answer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Answer for part 1: {}\nAnswer for part 2: {}",
            self.part_1,
            self.part_2
                .as_ref()
                .map_or_else(|| "unimplemented".to_string(), ToString::to_string)
        )
    }
}
