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

    // build custom alphabet with rare characters first
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

    let mut word_partitions: Vec<(&BitWord, u32)> = custom_words
        .iter()
        .map(|w| {
            (
                w,
                custom_alphabet
                    .chars()
                    .enumerate()
                    .find_map(|(i, _)| w.contains(i as u32).then(|| i as u32))
                    .unwrap(),
            )
        })
        .collect();
    word_partitions.sort_by_key(|(_, p)| *p);

    let frequency_sorted_words: Vec<BitWord> = word_partitions.iter().map(|(w, _)| **w).collect();

    // println!("word partitions:");
    // for w in frequency_sorted_words {
    //     println!("{} {:?}", w.format(&custom_alphabet), w);
    // }
    // return;

    let rare_chars = BitWord::encode(&custom_alphabet[..2].to_string(), &custom_alphabet);
    let rare_partition_index = frequency_sorted_words
        .iter()
        .position(|w| !w.has_overlap(&rare_chars))
        .unwrap();

    // build a list of all the words that contain at least one of the two least common characters
    // let rare_words: Vec<BitWord> = custom_words
    //     .iter()
    //     .filter(|x| x.has_overlap(&rare_chars))
    //     .map(|x| *x)
    //     .collect();

    // println!("rare words: {:?}", rare_words);
    println!("rare partition index: {:?}", rare_partition_index);
    // println!("number of rare words: {:?}", rare_words.len());
    println!("freq: {:?}", word_count_per_character);

    // println!("custom words:");
    // for word in &custom_words {
    //     println!(
    //         "{} {:?}",
    //         word.format(&custom_alphabet)
    //             .chars()
    //             .rev()
    //             .collect::<String>(),
    //         word
    //     );
    // }

    println!("all preparations {:?}", start.elapsed());
    // find all sets of size N that have no common bits
    let mut sets = find_set_par(
        &frequency_sorted_words[..rare_partition_index],
        &frequency_sorted_words,
    );
    sets.sort();
    sets.dedup();
    println!("number of sets {:?}", sets.len());
    // TODO: match items in the set to original words and print nicely, also list anagrams
}

fn find_set_par(first_words: &[BitWord], words: &[BitWord]) -> Vec<Vec<BitWord>> {
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
    let next_words = words
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
                let hits = find_set(&words[i + 1..], bits.merge(word), set);
                set.pop();
                hits
            })
            .collect()
    }
}
