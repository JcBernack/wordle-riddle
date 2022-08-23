use std::time::Instant;

use fnv::FnvHashMap;
use rayon::prelude::*;

use crate::bit_word::BitWord;

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

pub fn solve(start: Instant, lines: &Vec<String>) {
    assert!(
        WORD_LENGTH * WORD_COUNT <= 26,
        "the alphabet only has 26 letters!"
    );
    let mut words = lines
        .iter()
        // keep only words with 5 letters
        .filter(|line| line.len() == WORD_LENGTH as usize)
        .map(BitWord::new)
        // keep only words with 5 unique letters (no duplicate letters)
        .filter(|w| w.count() == WORD_LENGTH)
        .collect::<Vec<BitWord>>();
    // remove any duplicates in the bitwise representation (anagrams)
    words.sort();
    words.dedup();

    println!("{:?} cooked words", words.len());

    // build a lookup table mapping from a letter to all words containing that letter
    let words_per_letter = ('A'..='Z')
        .par_bridge()
        .map(|c| {
            (
                c,
                words.iter().filter(|w| w.contains(c)).collect::<Vec<_>>(),
            )
        })
        .collect::<FnvHashMap<_, _>>();

    // sort the buckets by the number of words per letter, meaning the first elements will be "rare" letters
    let mut word_count_per_letter = words_per_letter
        .iter()
        .map(|(c, w)| (*c, w.len()))
        .collect::<Vec<_>>();
    word_count_per_letter.sort_unstable_by_key(|(_, l)| *l);

    let rare_letter1 = word_count_per_letter[0].0;
    let rare_letter2 = word_count_per_letter[1].0;

    println!("freq: {:?}", word_count_per_letter);

    // build a list of all the words that contain at least one of the two least common letters
    let mut rare_words: Vec<BitWord> = words_per_letter[&rare_letter1]
        .iter()
        .chain(&words_per_letter[&rare_letter2])
        .map(|x| **x)
        .collect();
    rare_words.sort();
    rare_words.dedup();

    // println!("starting_words: {:?}", starting_words);
    println!("number of rare words: {:?}", rare_words.len());
    println!("{:?} frequency bucketing", start.elapsed());

    // let mut narf: Vec<BitWord> = encoded_words
    //     .iter()
    //     .enumerate()
    //     .flat_map(|(i, w1)| {
    //         encoded_words
    //             .iter()
    //             .skip(i + 1)
    //             .map(|w2| w1 | w2)
    //             .filter(|x| x.count_ones() == 10)
    //             .collect::<Vec<BitWord>>()
    //     })
    //     .collect();
    // println!("number of pairs: {}", narf.len());
    // narf.sort();
    // narf.dedup();
    // println!("number of unique pairs: {}", narf.len());
    // return;

    println!("number of encoded words {}", words.len());
    println!("words cooked in {:?}", start.elapsed());
    // let mut candidates = encoded_words.clone();
    // for i in 1..5 {
    //     println!("run {}", i);
    //     candidates = find_set_yolo(&candidates, &encoded_words);
    // }
    // println!("find pairs");
    // let pairs = find_set_yolo(&encoded_words, &encoded_words);
    // println!("find pairs of pairs");
    // let quaduplets = find_set_yolo(&pairs, &pairs);
    // println!("find pairs of 5er sets");
    // let fiver = find_set_yolo(&quaduplets, &encoded_words);

    // find all sets of size N that have no common bits
    let mut sets = find_set_par(&rare_words, &words);
    sets.sort();
    sets.dedup();
    println!("number of sets {:?}", sets.len());
    // let mut unique_pairs: Vec<BitWord> = sets
    //     .par_iter()
    //     .map(|set| set.iter().fold(0, |sum, w| sum | w))
    //     .collect();
    // unique_pairs.sort();
    // unique_pairs.dedup();
    // println!("number of unique pairs: {}", unique_pairs.len());
    // for set in sets {
    //     let x = set.iter().fold(0, |sum, w| sum | w);
    //     println!("{}", format_encoded_word(&x));
    // }
    // TODO: match items in the set to original words and print nicely, also list anagrams
    // println!(
    //     "found a set {:?}",
    //     set.iter().map(format_encoded_word).collect::<String>()
    // );
}

// fn find_set_yolo(candidates: &Vec<BitWord>, words: &Vec<BitWord>) -> Vec<BitWord> {
//     println!("number of inputs: {}", candidates.len());
//     let mut narf: Vec<BitWord> = candidates
//         .par_iter()
//         .enumerate()
//         .flat_map(|(i, w1)| {
//             words
//                 .iter()
//                 .skip(i + 1)
//                 .filter(|&w2| w1 & w2 == 0)
//                 .map(|w2| w1 | w2)
//                 .collect::<Vec<BitWord>>()
//         })
//         .collect();
//     narf.par_sort();
//     narf.dedup();
//     println!("number of outputs: {}", narf.len());
//     narf
// }

// TODO: return iterator instead of collect()'ing
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

// TODO: return iterator instead of collect()'ing
fn find_set(words: &[BitWord], bits: BitWord, set: &mut Vec<BitWord>) -> Vec<Vec<BitWord>> {
    // skip all words that have a lower numerical value than bits, they cannot be a match
    // use binary search to find the first word does not have a lower numerical value
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
