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

    fn empty(&self) -> bool {
        self.0 == 0
    }

    fn len(&self) -> u32 {
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

    let allowed = include_str!("../wordle-nyt-allowed-guesses.txt");
    let alphabetical = include_str!("../wordle-nyt-answers-alphabetical.txt");

    let (mut words, word_to_str): (Vec<_>, FnvHashMap<_, _>) = allowed
        .lines()
        .chain(alphabetical.lines())
        .map(|l| {
            let w = l
                .chars()
                .fold(Bitword::default(), |a, c| a | Bitword::new(c));
            (w, (w, l))
        })
        .filter(|(w, _)| w.len() == 5)
        .unzip();
    words.par_sort_unstable();
    words.dedup();

    let words_set = words.iter().collect::<FnvHashSet<_>>();

    // build a lookup table mapping from a letter to all words containing that letter
    let buckets = ('a'..='z')
        .map(|c| {
            (
                c,
                words
                    .iter()
                    .filter(|w| !(**w & Bitword::new(c)).empty())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<FnvHashMap<_, _>>();

    // sort the buckets by the number of words per letter, meaning the first elements will be "rare" letters
    let mut freq = buckets
        .iter()
        .map(|(c, words)| (*c, words.len()))
        .collect::<Vec<_>>();
    freq.sort_unstable_by_key(|(_, l)| *l);

    println!("freq: {:?}", freq);

    // build a list of all the words that contain at least one of the two least common letters
    let mut starting_words: Vec<&Bitword> = buckets[&freq[0].0]
        .iter()
        .chain(&buckets[&freq[1].0])
        .map(|x| *x)
        .collect();
    starting_words.sort();
    starting_words.dedup();

    // println!("starting_words: {:?}", starting_words);
    println!("number of starting_words: {:?}", starting_words.len());

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
                .filter(|w| w.len() == 10)
        })
        .collect();
    pairs.par_sort_unstable();
    pairs.dedup();

    let pairs_set = pairs.iter().collect::<FnvHashSet<_>>();

    println!("{} pairs found", pairs.len());

    // finds the first word w which is contained in the pair p and where the other 5 letters,
    // named x, of the pair make up a valid word too, returns both words w and x
    let pair_to_str = |p: Bitword| {
        words
            .iter()
            .find_map(|w| {
                // overlap 2 letters
                // p: 000001111111111
                // w: 111001100000000
                // ^: 111000011111111 => len 8
                // overlap 5 letters
                // p: 000001111111111
                // w: 000001111100000
                // ^: 000000000011111 => len 5
                let x = p ^ *w;
                (x.len() == 5 && words_set.contains(&x)).then(|| (word_to_str[w], word_to_str[&x]))
            })
            .unwrap()
    };

    let solutions = pairs
        .par_iter()
        .filter(|pair| {
            // println!("{:?}", **w);
            (**pair & Bitword::new(freq[0].0)).empty() && (**pair & Bitword::new(freq[1].0)).empty()
        })
        .flat_map_iter(|pair| {
            starting_words
                .iter()
                .map(|&sw| (sw, *sw | *pair))
                .filter(|(_, combined)| combined.len() == 15)
                .flat_map(|(sw, combined)| {
                    let end = !combined;
                    end.chars()
                        .map(move |c| end ^ Bitword::new(c))
                        .filter(|e| pairs_set.contains(&e))
                        .map(move |e| (*sw, *pair, e))
                })
                .collect::<Vec<(Bitword, Bitword, Bitword)>>()
        })
        .map(|(w, p0, p1)| {
            let (w1, w2) = pair_to_str(p0);
            let (w3, w4) = pair_to_str(p1);
            let mut s = [word_to_str[&w], w1, w2, w3, w4];
            s.sort_unstable();
            s
        })
        .collect::<FnvHashSet<_>>();

    println!("{:?} elapsed", start.elapsed());
    println!("{} solutions {:?}", solutions.len(), solutions);
}
