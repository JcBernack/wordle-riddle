use std::{
    cmp::Ordering,
    fmt::Debug,
    time::Instant,
};

use fnv::{FnvHashMap, FnvHashSet};
use rayon::{
    prelude::{IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

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
        write!(f, "{}", s.iter().collect::<String>())
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

fn union<T: Eq + Ord + Copy>(a: &[T], b: &[T]) -> Vec<T> {
    let mut vec = Vec::with_capacity(a.len().max(b.len()));
    for i in 0.. {
        match (a.get(i), b.get(i)) {
            (None, None) => break,
            (None, Some(v)) | (Some(v), None) => vec.push(*v),
            (Some(x), Some(y)) => match x.cmp(y) {
                Ordering::Less => {
                    vec.push(*x);
                    vec.push(*y);
                }
                Ordering::Equal => vec.push(*x),
                Ordering::Greater => {
                    vec.push(*y);
                    vec.push(*x);
                }
            },
        };
    }
    vec
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

    let words_set = words.iter().copied().collect::<FnvHashSet<_>>();

    let buckets = ('a'..='z')
        .map(|c| {
            (
                c,
                words
                    .iter()
                    .filter(|w| !(**w & Bitword::new(c)).empty())
                    .copied()
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<FnvHashMap<_, _>>();

    let mut freq = buckets
        .iter()
        .map(|(c, words)| (*c, words.len()))
        .collect::<Vec<_>>();
    freq.sort_unstable_by_key(|(_, l)| *l);

    // println!("freq: {:?}", freq);

    let mut pairs = words
        .par_iter()
        .fold(Vec::new, |mut vec, word| {
            words
                .iter()
                .map(|w| *w | *word)
                .filter(|w| w.len() == 10)
                .for_each(|w| vec.push(w));
            vec
        })
        .reduce(Vec::new, |mut a, mut v| {
            a.append(&mut v);
            a
        });
    pairs.par_sort_unstable();
    pairs.dedup();

    let pairs_set = pairs.iter().copied().collect::<FnvHashSet<_>>();

    // println!("{} pairs found", pairs.len());

    let pair_to_str = |p: Bitword| {
        words
            .iter()
            .find_map(|w| {
                let x = p ^ *w;
                (x.len() == 5 && words_set.contains(&x)).then(|| (word_to_str[w], word_to_str[&x]))
            })
            .unwrap()
    };

    let starting_words = union(&buckets[&freq[0].0], &buckets[&freq[1].0]);

    let solutions = pairs
        .par_iter()
        .filter(|w| {
            // println!("{:?}", **w);
            (**w & Bitword::new(freq[0].0)).empty() && (**w & Bitword::new(freq[1].0)).empty()
        })
        .fold(Vec::new, |mut vec, w| {
            starting_words.iter().for_each(|sw| {
                let combined = *w | *sw;
                if combined.len() == 15 {
                    let end = !combined;
                    for c in end.chars() {
                        let e = end ^ Bitword::new(c);
                        if pairs_set.contains(&e) {
                            vec.push((*sw, *w, e))
                        }
                    }
                }
            });
            vec
        })
        .reduce(Vec::new, |mut a, mut v| {
            a.append(&mut v);
            a
        })
        .iter()
        .map(|(w, p0, p1)| {
            let (w1, w2) = pair_to_str(*p0);
            let (w3, w4) = pair_to_str(*p1);
            let mut s = [word_to_str[w], w1, w2, w3, w4];
            s.sort_unstable();
            s
        })
        .collect::<FnvHashSet<_>>();

    println!("{:?} elapsed", start.elapsed());
    println!("{} solutions {:?}", solutions.len(), solutions);
}
