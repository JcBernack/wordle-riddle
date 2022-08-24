use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::time::Instant;

use clap::Parser;
use clap::Subcommand;
use num_bigint::{BigUint, ToBigUint};

mod bernack;
mod bit_word;
mod bored_person;

#[derive(Parser)]
// #[clap(author, version, about, long_about = None)]
struct Cli {
    /// Filename to read words from
    #[clap(value_parser)]
    filename: String,

    /// Number of letters per word, this will filter the given word list
    #[clap(short, long, default_value_t = 5)]
    letters: u32,

    /// Number of words to find
    #[clap(short, long, default_value_t = 5)]
    words: u32,

    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// run the algorithm by JcBernack
    Bernack,

    /// run the algorithm by BoredPerson
    Bored,
}

fn main() {
    let start = Instant::now();
    let cli: Cli = Cli::parse();

    assert!(
        cli.words * cli.letters <= 26,
        "the alphabet only has 26 characters!"
    );

    let lines: Vec<String> = match read_file(cli.filename) {
        Err(e) => panic!("unable to load words: {}", e),
        Ok(reader) => reader
            .lines()
            .map(|s| s.unwrap())
            // keep only words with N characters
            .filter(|l| l.len() == cli.letters as usize)
            .collect(),
    };

    println!("{:?} input file read", start.elapsed());
    println!("{:?} words read", lines.len());
    println!(
        "{} possible combinations",
        big_bionmial(lines.len() as u32, 5)
    );

    match &cli.command {
        Some(Commands::Bernack) => {
            println!("run algorithm by @JcBernack");
            bernack::solve(start, &lines);
        }
        Some(Commands::Bored) => {
            println!("run algorithm by @BoredPerson");
            bored_person::solve(start, &lines);
        }
        None => {
            println!("please provide an algorithm to run");
        }
    }

    println!("{:?} done", start.elapsed());
}

fn big_bionmial(n: u32, k: u32) -> BigUint {
    let mut res = BigUint::from(1 as u32);
    for i in 0..k {
        res = (res * (n - i).to_biguint().unwrap()) / (i + 1).to_biguint().unwrap();
    }
    res
}

// The output is wrapped in a Result to allow matching on errors
fn read_file<P>(filename: P) -> io::Result<BufReader<File>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(BufReader::new(file))
}
