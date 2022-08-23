use std::time::Instant;

use fnv::FnvHashMap;
use rayon::prelude::*;

use crate::bit_word::BitWord;

/// Solution times for words_alpha.txt:
/// completed in 187.500069011s first attempt
/// completed in  99.903330992s all loops parallel
/// completed in  23.887663784s outermost loop in parallel
/// completed in  15.566494685s remove redundant set copying in the non-parallelized loops
/// completed in  10.085339854s replace range with enumerate()
/// completed in  18.833595413s collect all matching sets (number of hits 538)
/// completed in  17.719814862s skip by numerical value
/// completed in  13.953842087s skip by numerical value with binary search
/// completed in  13.766282412s skip via slice not iterator
/// completed in   7.686388088s start with word including at least one of the rarest characters

const WORD_LENGTH: u32 = 5;
const WORD_COUNT: u32 = 5;

pub fn solve(start: Instant, lines: &Vec<String>) {
    assert!(
        WORD_LENGTH * WORD_COUNT <= 26,
        "the alphabet only has 26 characters!"
    );
    let mut words = lines
        .iter()
        // keep only words with 5 characters
        .filter(|line| line.len() == WORD_LENGTH as usize)
        .map(BitWord::new)
        // keep only words with 5 unique characters (no duplicate characters)
        .filter(|w| w.count() == WORD_LENGTH)
        .collect::<Vec<BitWord>>();
    // remove any duplicates in the bitwise representation (anagrams)
    words.sort();
    words.dedup();

    println!("number of encoded words {}", words.len());
    println!("words cooked in {:?}", start.elapsed());

    // build a lookup table mapping from a character to all words containing that character
    let words_per_character = ('A'..='Z')
        .par_bridge()
        .map(|c| {
            (
                c,
                words.iter().filter(|w| w.contains(c)).collect::<Vec<_>>(),
            )
        })
        .collect::<FnvHashMap<_, _>>();

    // sort the buckets by the number of words per character, meaning the first elements will be "rare" characters
    let mut word_count_per_character = words_per_character
        .iter()
        .map(|(c, w)| (*c, w.len()))
        .collect::<Vec<_>>();
    word_count_per_character.sort_unstable_by_key(|(_, l)| *l);

    let rare_character1 = word_count_per_character[0].0;
    let rare_character2 = word_count_per_character[1].0;

    // build a list of all the words that contain at least one of the two least common characters
    let mut rare_words: Vec<BitWord> = words_per_character[&rare_character1]
        .iter()
        .chain(&words_per_character[&rare_character2])
        .map(|x| **x)
        .collect();
    rare_words.sort();
    rare_words.dedup();

    // println!("rare words: {:?}", rare_words);
    println!("number of rare words: {:?}", rare_words.len());
    println!("freq: {:?}", word_count_per_character);
    println!("frequency bucketing {:?}", start.elapsed());

    // find all sets of size N that have no common bits
    let mut sets = find_set_par(&rare_words, &words);
    sets.sort();
    sets.dedup();
    println!("number of sets {:?}", sets.len());
    // TODO: match items in the set to original words and print nicely, also list anagrams
}

fn find_set_par(first_words: &Vec<BitWord>, words: &Vec<BitWord>) -> Vec<Vec<BitWord>> {
    (0..first_words.len())
        .into_par_iter()
        .flat_map(|i| {
            if i % 100 == 0 {
                println!("{}/{}", i, first_words.len());
            }
            find_set(&words, first_words[i], &mut vec![first_words[i]])
        })
        .collect()
}

fn find_set(words: &[BitWord], bits: BitWord, set: &mut Vec<BitWord>) -> Vec<Vec<BitWord>> {
    // skip all words that have a lower numerical value than bits, they cannot be a match
    // use binary search to find the first word does not have a lower numerical value
    // TODO: cannot use skipping currently because the rare_word's mess up with the order
    let skipped = 0; //words.partition_point(|&x| x < bits);
    let next_words = words[skipped..]
        .iter()
        .enumerate()
        // only keep words that have no overlap with bits
        .filter(|(_, word)| !bits.has_overlap(word));
    if set.len() + 1 == WORD_COUNT as usize {
        next_words
            .map(|(_, word)| {
                let mut hit = set.clone();
                hit.push(*word);
                hit.sort();
                return hit;
            })
            .collect()
    } else {
        next_words
            .flat_map(|(i, word)| {
                set.push(*word);
                let hits = find_set(&words[skipped + i + 1..], bits.merge(word), set);
                set.pop();
                hits
            })
            .collect()
    }
}
