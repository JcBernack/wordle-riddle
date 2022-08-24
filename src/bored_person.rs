use std::time::Instant;

use fnv::{FnvHashMap, FnvHashSet};
use rayon::prelude::ParallelBridge;
use rayon::prelude::*;
use rayon::slice::ParallelSliceMut;

use crate::bit_word::BitWord;

pub fn solve(start: Instant, lines: &Vec<String>) {

    let alphabet: String = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string();

    let (mut words, word_to_str): (Vec<_>, FnvHashMap<_, _>) = lines
        .iter()
        .filter(|line| line.len() == 5)
        .map(|line| {
            let w = BitWord::encode(line, &alphabet);
            (w, (w, line))
        })
        .filter(|(w, _)| w.count() == 5)
        .unzip();
    words.par_sort_unstable();
    words.dedup();

    let words_set = words.iter().collect::<FnvHashSet<_>>();

    println!("{:?} cooking words", start.elapsed());
    println!("{:?} cooked words", words.len());

    // build a lookup table mapping from a character to all words containing that character
    let words_per_character = alphabet
        .chars()
        .enumerate()
        .par_bridge()
        .map(|(i, c)| {
            (
                c,
                words.iter().filter(|w| w.contains(i as u32)).collect::<Vec<_>>(),
            )
        })
        .collect::<FnvHashMap<_, _>>();

    // sort the buckets by the number of words per character, meaning the first elements will be "rare" characters
    let mut word_count_per_character = words_per_character
        .iter()
        .map(|(c, words)| (*c, words.len()))
        .collect::<Vec<_>>();
    word_count_per_character.sort_unstable_by_key(|(_, l)| *l);

    let rare_character1 = word_count_per_character[0].0;
    let rare_character2 = word_count_per_character[1].0;

    println!("freq: {:?}", word_count_per_character);

    // build a list of all the words that contain at least one of the two least common characters
    let mut rare_words: Vec<&BitWord> = words_per_character[&rare_character1]
        .iter()
        .chain(&words_per_character[&rare_character2])
        .map(|x| *x)
        .collect();
    rare_words.sort();
    rare_words.dedup();

    // println!("starting_words: {:?}", starting_words);
    println!("number of rare words: {:?}", rare_words.len());
    println!("{:?} frequency bucketing", start.elapsed());

    // build unique pairs of words with 10 unique characters
    let mut pairs: Vec<BitWord> = words
        .iter()
        .enumerate()
        .par_bridge()
        .flat_map_iter(|(i, word)| {
            words
                .iter()
                .skip(i)
                .map(|w| w.merge(word))
                .filter(|w| w.count() == 10)
        })
        .collect();
    pairs.par_sort_unstable();
    pairs.dedup();

    let pairs_set = pairs.iter().collect::<FnvHashSet<_>>();

    println!("{:?} finding pairs", start.elapsed());
    println!("{} pairs found", pairs.len());

    // finds all words w1 which are contained in the pair p and where the other 5 characters
    // of the pair make up a valid word w2 too, returns a list of pairs (w1, w2)
    let words_in_pair = |p: BitWord| {
        words
            .iter()
            .filter(|&w| p.flip(w).count() == 5)
            .map(|w| {
                // overlap 2 characters
                // p: 000001111111111
                // w: 111001100000000
                // ^: 111000011111111 => len 8 => not a word
                // overlap 5 characters
                // p: 000001111111111
                // w: 000001111100000
                // ^: 000000000011111 => len 5 => valid other word in the pair
                (*w, p.flip(w))
            })
            .filter(|(_, w2)| words_set.contains(&w2))
            .map(|(w1, w2)| (word_to_str[&w1], word_to_str[&w2]))
            .collect::<Vec<(&String, &String)>>()
    };

    // match pairs to a word containing at least one of the two rarest characters
    // then determine the remaining 10 characters and check if a pair using those exists
    let solutions = pairs
        .par_iter()
        // .filter(|pair| {
        //     // println!("{:?}", **w);
        //     !pair.contains(rare_character1) || !pair.contains(rare_character2)
        // })
        .flat_map_iter(|pair| {
            rare_words
                .iter()
                .map(|&sw| (sw, sw.merge(pair)))
                .filter(|(_, combined)| combined.count() == 15)
                .flat_map(|(sw, combined)| {
                    // end will have all bits set of the 11 remaining, yet unused characters
                    let remaining = combined.invert(&alphabet);
                    remaining
                        .bits()
                        // individually toggle each remaining bit off, resulting in 10 bit words
                        .map(move |i| remaining.toggle(i))
                        // and try to find a pair using those 10 characters
                        .filter(|e| pairs_set.contains(&e))
                        // if there is one we can complete our 15 character set with the 10 characters of this pair
                        .map(move |e| (*sw, *pair, e))
                })
                .collect::<Vec<(BitWord, BitWord, BitWord)>>()
        })
        .flat_map_iter(|(w, p0, p1)| {
            // map back to strings
            let w0 = &word_to_str[&w];
            let wp0 = words_in_pair(p0);
            let wp1 = words_in_pair(p1);
            wp0.iter()
                .flat_map(|(w1, w2)| {
                    wp1.iter().map(|(w3, w4)| {
                        let mut set = [*w0, *w1, *w2, *w3, *w4];
                        set.sort();
                        set
                    })
                })
                .collect::<Vec<[&String; 5]>>()
        })
        .collect::<FnvHashSet<_>>();

    // println!("solutions {:?}", solutions);
    println!("number of solutions {}", solutions.len());
    println!("{:?} total time", start.elapsed());
}
