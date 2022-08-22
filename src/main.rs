mod bored_person;

use rayon::prelude::*;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::time::Instant;

/// Results:
/// completed in 187.500069011s first attempt
/// completed in 99.903330992s all loops parallel
/// completed in 23.887663784s outermost loop in parallel
/// completed in 15.566494685s remove redundant set copying in the non-parallelized loops
/// completed in 10.085339854s replace range with enumerate()
/// completed in 9.68340491s
/// completed in 18.833595413s collect all matching sets (number of hits 538)
/// completed in 17.719814862s skip by numerical value
/// completed in 13.953842087s skip by numerical value with binary search
/// completed in 13.766282412s skip via slice not iterator

/// Different approach idea:
/// observations:
/// - the more words are in the set, the more bits are set
/// - the more bits are set the higher the chances of "duplicate search paths"
/// - try to merge these to reduce work
///
/// - this is similar to the removal of anagrams as one of the first steps
/// - anagrams have identical search paths in this problem
/// - the approach is basically to find and merge "set anagrams" without losing all the information about which words were used to build them
///
/// mapping of binary representation to a structure holding words
/// sort and merge such that the binaries are unique afterwards and
/// all the words of duplicate binaries are merged into one
///
/// before
/// ABCDE--------------------- abcde
/// ABCDE--------------------- abced (anagram)
/// ABC--FG------------------- abcfg
/// ---DE--HIJ---------------- dehij
/// -----FGHIJ---------------- fghij
///
/// after
/// ABCDE--------------------- (abcde, abced)
/// ABC--FG------------------- (abcfg)
/// ---DE--HIJ---------------- (dehij)
/// -----FGHIJ---------------- (fghij)
///
/// then build all combinations of entries which have no overlap: (a & b == 0)
/// while also merging their result structure
/// ABCDEFGHIJ---------------- ((abcde, abced), (fghij)) combination of 1 and 4
/// ABCDEFGHIJ---------------- ((abcfg), (dehij)) combination of 2 and 3
///
/// then sort and merge by binary representation again
/// ABCDEFGHIJ---------------- (((abcde, abced), (fghij)), ((abcfg), (dehij)))
///
/// repeat 5 times
///

const WORD_LENGTH: u32 = 5;
const WORD_COUNT: u32 = 5;

type Word = u32;

fn main() {
    // bored_person::main();
    // return;
    assert!(
        WORD_LENGTH * WORD_COUNT <= 26,
        "the alphabet only has 26 letters!"
    );
    let start = Instant::now();
    match read_lines("./words_alpha.txt") {
        Err(e) => println!("unable to load words: {}", e),
        Ok(lines) => {
            let mut encoded_words = lines
                .map(|x| x.unwrap())
                // keep only words with 5 letters
                .filter(|x| x.len() == WORD_LENGTH as usize)
                .flat_map(encode_word)
                // keep only words with 5 unique letters (no duplicate letters)
                .filter(|x| x.count_ones() == WORD_LENGTH)
                .collect::<Vec<Word>>();
            // remove any duplicates in the bitwise representation (anagrams)
            encoded_words.sort();
            encoded_words.dedup();
            // encoded_words.reverse();
            // for x in &encoded_words {
            //     println!("{} {}", format_encoded_word(x), x);
            // }

            // let mut narf: Vec<Word> = encoded_words
            //     .iter()
            //     .enumerate()
            //     .flat_map(|(i, w1)| {
            //         encoded_words
            //             .iter()
            //             .skip(i + 1)
            //             .map(|w2| w1 | w2)
            //             .filter(|x| x.count_ones() == 10)
            //             .collect::<Vec<Word>>()
            //     })
            //     .collect();
            // println!("number of pairs: {}", narf.len());
            // narf.sort();
            // narf.dedup();
            // println!("number of unique pairs: {}", narf.len());
            // return;

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
            let sets = find_set_par(&encoded_words);
            println!("number of sets {:?}", sets.len());
            // TODO: match items in the set to original words and print nicely, also list anagrams
            // println!(
            //     "found a set {:?}",
            //     set.iter().map(format_encoded_word).collect::<String>()
            // );
        }
    }
    println!("completed in {:?}", start.elapsed());
}

// TODO: return iterator instead of collect()'ing
fn find_set_par(words: &Vec<Word>) -> Vec<Vec<Word>> {
    (0..words.len())
        .into_par_iter()
        .flat_map(|i| {
            if i % 1000 == 0 {
                println!("{}/{}", i, words.len());
            }
            find_set(&words[i + 1..], words[i], &mut vec![words[i]])
        })
        .collect()
}

// TODO: return iterator instead of collect()'ing
fn find_set(words: &[u32], bits: Word, set: &mut Vec<Word>) -> Vec<Vec<Word>> {
    // skip all words that have a lower numerical value than bits, they cannot be a match
    // use binary search to find the first word with a numerical value larger or equal to bits
    let skipped = words.partition_point(|&x| x < bits);
    let next_sets = words[skipped..]
        .iter()
        .enumerate()
        // only keep words that have no overlap with bits
        .filter(|(_, word)| *word & bits == 0);
    if set.len() + 1 == WORD_COUNT as usize {
        next_sets
            .map(|(_, word)| {
                let mut hit = set.clone();
                hit.push(*word);
                return hit;
            })
            .collect()
    } else {
        next_sets
            .flat_map(|(i, word)| {
                set.push(*word);
                let hits = find_set(&words[skipped + i + 1..], word | bits, set);
                set.pop();
                hits
            })
            .collect()
    }
}

// example output: "----E---I-----O-------W--Z"
fn format_encoded_word(bits: &Word) -> String {
    (0..26)
        .flat_map(|i| match bits >> i & 1 == 1 {
            true => char::from_u32(i + 'A' as u32),
            false => Some('-'),
        })
        .collect()
}

fn encode_word(word: String) -> Option<Word> {
    word.to_ascii_uppercase()
        .chars()
        .map(|c| 1 << (c as u32 - 'A' as u32))
        .reduce(|sum, x| sum | x)
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
