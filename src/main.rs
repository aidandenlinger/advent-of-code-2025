//! Advent of Code driver
use std::{fs::read_to_string, path::PathBuf};

use clap::{Parser, value_parser};
use color_eyre::eyre::{Context, Result};

mod day01;

#[derive(Parser)]
#[command(about)]
struct Cli {
    /// The day that should be run. Will run part 1 and part 2.
    #[arg(value_parser = value_parser!(u8).range(1..=12))]
    day: u8,

    /// Folder holding input files. "day01.txt" for day 1, "day02.txt" for
    /// day 2, etc.
    #[arg(default_value = "./input")]
    input_folder: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;
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
