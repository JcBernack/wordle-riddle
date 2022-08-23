use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::time::Instant;

mod bit_word;
mod bored_person;
mod gigo;

fn main() {
    let start = Instant::now();
    match read_file("./words_alpha.txt") {
        Err(e) => println!("unable to load words: {}", e),
        Ok(reader) => {
            let lines: Vec<String> = reader
                .lines()
                .map(|s| s.unwrap())
                .collect();
            bored_person::solve(start, &lines);
            println!("------------------------------");
            gigo::solve(start, &lines);
        }
    }
    println!("completed in {:?}", start.elapsed());
}

// The output is wrapped in a Result to allow matching on errors
fn read_file<P>(filename: P) -> io::Result<BufReader<File>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(BufReader::new(file))
}
