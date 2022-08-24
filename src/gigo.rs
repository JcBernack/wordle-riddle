use std::time::Instant;

use fnv::FnvHashMap;
use rayon::prelude::*;

use crate::bit_word::BitWord;

const WORD_LENGTH: u32 = 5;
const WORD_COUNT: u32 = 5;

pub fn solve(start: Instant, lines: &Vec<String>) {
    assert!(
        WORD_LENGTH * WORD_COUNT <= 26,
        "the alphabet only has 26 characters!"
    );

    let alphabet: String = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string();

    let mut words = lines
        .iter()
        // keep only words with 5 characters
        .filter(|line| line.len() == WORD_LENGTH as usize)
        .map(|line| BitWord::encode(line, &alphabet))
        // keep only words with 5 unique characters (no duplicate characters)
        .filter(|w| w.count() == WORD_LENGTH)
        .collect::<Vec<BitWord>>();
    // remove any duplicates in the bitwise representation (anagrams)
    words.sort();
    words.dedup();

    println!("number of encoded words {}", words.len());
    println!("words encoded in {:?}", start.elapsed());
    // for word in &words {
    //     println!("{}", word.format(&alphabet));
    // }

    // build a lookup table mapping from a character to all words containing that character
    let words_per_character = alphabet
        .chars()
        .enumerate()
        .par_bridge()
        .map(|(i, c)| {
            (
                c,
                words
                    .iter()
                    .filter(|w| w.contains(i as u32))
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<FnvHashMap<_, _>>();

    // sort the buckets by the number of words per character, meaning the first elements will be "rare" characters
    let mut word_count_per_character = words_per_character
        .iter()
        .map(|(c, w)| (*c, w.len()))
        .collect::<Vec<_>>();
    word_count_per_character.sort_unstable_by_key(|(_, l)| *l);

    let custom_alphabet: String = word_count_per_character.iter().map(|(c, _)| c).collect();

    println!("custom alphabet: {}", custom_alphabet);

    // reencode all words using new alphabet
    let mut custom_words = lines
        .iter()
        // keep only words with 5 characters
        .filter(|line| line.len() == WORD_LENGTH as usize)
        .map(|line| BitWord::encode(line, &custom_alphabet))
        // keep only words with 5 unique characters (no duplicate characters)
        .filter(|w| w.count() == WORD_LENGTH)
        .collect::<Vec<BitWord>>();
    // remove any duplicates in the bitwise representation (anagrams)
    custom_words.sort();
    custom_words.dedup();

    let rare_chars = BitWord::encode(&custom_alphabet[..2].to_string(), &custom_alphabet);

    // build a list of all the words that contain at least one of the two least common characters
    let rare_words: Vec<BitWord> = custom_words
        .iter()
        .filter(|x| x.has_overlap(&rare_chars))
        .map(|x| *x)
        .collect();

    // println!("rare words: {:?}", rare_words);
    println!("number of rare words: {:?}", rare_words.len());
    println!("freq: {:?}", word_count_per_character);
    println!("frequency bucketing {:?}", start.elapsed());

    println!("custom words:");
    for word in &custom_words {
        println!(
            "{} {:?}",
            word.format(&custom_alphabet)
                .chars()
                .rev()
                .collect::<String>(),
            word
        );
    }

    // find all sets of size N that have no common bits
    let mut sets = find_set_par(&rare_words, &custom_words);
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
    // TODO: cannot use skipping currently because the rare_word's mess up the order
    let skipped = words.partition_point(|&x| x < bits);
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
