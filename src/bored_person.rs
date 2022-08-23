use std::{fmt::Debug, time::Instant};

use fnv::{FnvHashMap, FnvHashSet};
use rayon::prelude::ParallelBridge;
use rayon::prelude::*;
use rayon::slice::ParallelSliceMut;

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Bitword(u32);

impl Bitword {
    fn new(c: char) -> Self {
        Self(1 << ((c as u32) - ('a' as u32)))
    }

    fn encode(word: &str) -> Self {
        word.chars()
            .fold(Bitword::default(), |bits, c| bits | Bitword::new(c))
    }

    fn empty(&self) -> bool {
        self.0 == 0
    }

    fn contains(&self, c: char) -> bool {
        !(*self & Bitword::new(c)).empty()
    }

    fn letters(&self) -> u32 {
        self.0.count_ones()
    }

    fn chars(&self) -> impl Iterator<Item = char> {
        let mut x = self.0;
        std::iter::from_fn(move || {
            let zeros = x.trailing_zeros();
            (zeros < u32::BITS).then(|| {
                char::from_u32({
                    x ^= 1 << zeros;
                    ('a' as u32) + zeros
                })
                .unwrap()
            })
        })
    }
}

impl Debug for Bitword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = ['-'; 26];
        for i in 0..26 {
            if (self.0 & (1 << i)) != 0 {
                s[i as usize] = char::from_u32(('a' as u32) + i).unwrap();
            }
        }
        write!(f, "\n{}", s.iter().collect::<String>())
    }
}

impl std::ops::BitOr for Bitword {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for Bitword {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitXor for Bitword {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::Not for Bitword {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0 & ((1 << 26) - 1))
    }
}

pub fn main() {
    let start = Instant::now();

    let data = include_str!("../words_alpha.txt");

    let (mut words, word_to_str): (Vec<_>, FnvHashMap<_, _>) = data
        .lines()
        .filter(|w| w.len() == 5)
        .map(|l| {
            let w = Bitword::encode(l);
            (w, (w, l))
        })
        .filter(|(w, _)| w.letters() == 5)
        .unzip();
    words.par_sort_unstable();
    words.dedup();

    let words_set = words.iter().collect::<FnvHashSet<_>>();

    println!("{:?} cooking words", start.elapsed());
    println!("{:?} cooked words", words.len());

    // build a lookup table mapping from a letter to all words containing that letter
    let words_per_letter = ('a'..='z')
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
        .map(|(c, words)| (*c, words.len()))
        .collect::<Vec<_>>();
    word_count_per_letter.sort_unstable_by_key(|(_, l)| *l);

    let rare_letter1 = word_count_per_letter[0].0;
    let rare_letter2 = word_count_per_letter[1].0;

    println!("freq: {:?}", word_count_per_letter);

    // build a list of all the words that contain at least one of the two least common letters
    let mut rare_words: Vec<&Bitword> = words_per_letter[&rare_letter1]
        .iter()
        .chain(&words_per_letter[&rare_letter2])
        .map(|x| *x)
        .collect();
    rare_words.sort();
    rare_words.dedup();

    // println!("starting_words: {:?}", starting_words);
    println!("number of rare words: {:?}", rare_words.len());
    println!("{:?} frequency bucketing", start.elapsed());

    // build unique pairs of words with 10 unique letters
    let mut pairs: Vec<Bitword> = words
        .iter()
        .enumerate()
        .par_bridge()
        .flat_map_iter(|(i, word)| {
            words
                .iter()
                .skip(i)
                .map(|w| *w | *word)
                .filter(|w| w.letters() == 10)
        })
        .collect();
    pairs.par_sort_unstable();
    pairs.dedup();

    let pairs_set = pairs.iter().collect::<FnvHashSet<_>>();

    println!("{:?} finding pairs", start.elapsed());
    println!("{} pairs found", pairs.len());

    // finds all words w1 which are contained in the pair p and where the other 5 letters
    // of the pair make up a valid word w2 too, returns a list of pairs (w1, w2)
    let words_in_pair = |p: Bitword| {
        words
            .iter()
            .filter(|&w| (p ^ *w).letters() == 5)
            .map(|w| {
                // overlap 2 letters
                // p: 000001111111111
                // w: 111001100000000
                // ^: 111000011111111 => len 8 => not a word
                // overlap 5 letters
                // p: 000001111111111
                // w: 000001111100000
                // ^: 000000000011111 => len 5 => valid other word in the pair
                (*w, p ^ *w)
            })
            .filter(|(_, w2)| words_set.contains(&w2))
            .map(|(w1, w2)| (word_to_str[&w1], word_to_str[&w2]))
            .collect::<Vec<(&str, &str)>>()
    };

    // match pairs to a word containing at least one of the two rarest letters
    // then determine the remaining 10 letters and check if a pair using those exists
    let solutions = pairs
        .par_iter()
        // .filter(|pair| {
        //     // println!("{:?}", **w);
        //     !pair.contains(rare_letter1) || !pair.contains(rare_letter2)
        // })
        .flat_map_iter(|pair| {
            rare_words
                .iter()
                .map(|&sw| (sw, *sw | *pair))
                .filter(|(_, combined)| combined.letters() == 15)
                .flat_map(|(sw, combined)| {
                    // end will have all bits set of the 11 remaining, yet unused letters
                    let remaining = !combined;
                    remaining
                        .chars()
                        // individually toggle each remaining bit off, resulting in 10 bit words
                        .map(move |c| remaining ^ Bitword::new(c))
                        // and try to find a pair using those 10 letters
                        .filter(|e| pairs_set.contains(&e))
                        // if there is one we can complete our 15 letter set with the 10 letters of this pair
                        .map(move |e| (*sw, *pair, e))
                })
                .collect::<Vec<(Bitword, Bitword, Bitword)>>()
        })
        .flat_map_iter(|(w, p0, p1)| {
            // map back to strings
            let w0 = word_to_str[&w];
            let wp0 = words_in_pair(p0);
            let wp1 = words_in_pair(p1);
            wp0.iter()
                .flat_map(|(w1, w2)| {
                    wp1.iter().map(|(w3, w4)| {
                        let mut set = [w0, w1, w2, w3, w4];
                        set.sort();
                        set
                    })
                })
                .collect::<Vec<[&str; 5]>>()
        })
        .collect::<FnvHashSet<_>>();

    // println!("solutions {:?}", solutions);
    println!("number of solutions {}", solutions.len());
    println!("{:?} total time", start.elapsed());
}
