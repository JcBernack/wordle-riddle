use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::time::Instant;

const WORD_LENGTH: u32 = 5;
// const WORD_COUNT: u32 = 5;

// TODO: do everything in uppercase, easier to read

fn main() {
    let start = Instant::now();
    match read_lines("./words_alpha.txt") {
        Err(e) => println!("unable to load words: {}", e),
        Ok(lines) => {
            let mut encoded_words = lines
                .map(|x| x.unwrap())
                // keep only words with 5 letters
                .filter(|x| x.len() == WORD_LENGTH as usize)
                .flat_map(encode_word)
                // keep only words with 5 unique letters (no duplicates)
                .filter(|x| x.count_ones() == WORD_LENGTH)
                .collect::<Vec<u32>>();
            // remove any duplicates in the bitwise representation (anagrams)
            encoded_words.sort();
            encoded_words.dedup();
            // encoded_words.reverse();
            for x in &encoded_words {
                println!("{} {}", format_encoded_word(x), x);
            }
            // println!(
            //     "encoded words {:?}",
            //     encoded_words
            //         .iter()
            //         .map(format_encoded_word)
            //         .collect::<Vec<String>>()
            // );
            println!("number of encoded words {}", encoded_words.len());
            println!("words cooked in {:?}", start.elapsed());
            // find all sets of size N that have no common bits
            let mut set: Vec<u32> = Vec::new();
            find_set(&encoded_words, 0, 0, &mut set);
            println!("completed in {:?}", start.elapsed());
        }
    }
}

fn find_set(items: &Vec<u32>, offset: usize, bits: u32, set: &mut Vec<u32>) {
    // TODO: parallelize loop, maybe just the outer loop?
    for i in offset..items.len() {
        let item = items[i];
        if set.len() == 0 && i % 100 == 0 {
            println!("{}/{}", i, items.len());
        }
        if item & bits == 0 {
            set.push(item.clone());
            if set.len() == 5 {
                // TODO: collect and return all sets, as iterator?
                // TODO: match items in the set to original words and print nicely, also list anagrams
                println!(
                    "found a set {:?}",
                    set.iter()
                        .map(format_encoded_word)
                        .collect::<String>()
                );
            } else {
                find_set(items, i + 1, item | bits, set);
            }
            set.pop();
        }
    }
}

// example output: "----e---i-----o-------w--z"
fn format_encoded_word(bits: &u32) -> String {
    (0..26)
        .flat_map(|i| match bits >> i & 1 == 1 {
            true => char::from_u32(i + 97),
            false => Some('-'),
        })
        .collect()
}

fn encode_word(word: String) -> Option<u32> {
    word.chars().map(char2bit).reduce(|sum, x| sum | x)
}

fn char2bit(c: char) -> u32 {
    1 << (c as u32 - 97)
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
