#![allow(dead_code)]

use std::fmt::Debug;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BitWord(u32);

impl BitWord {
    // encode word using the given alphabet
    pub fn encode(word: &String, alphabet: &String) -> Self {
        word.chars().fold(Self::default(), |word, c| {
            word.toggle(alphabet.find(c.to_ascii_uppercase()).unwrap() as u32)
        })
    }

    // decode word using the given alphabet
    pub fn format(&self, alphabet: &String) -> String {
        // example output: "----E---I-----O-------W--Z"
        alphabet
            .chars()
            .enumerate()
            .map(|(i, c)| match self.contains(i as u32) {
                true => c,
                false => '-',
            })
            .collect()
    }

    pub fn toggle(&self, i: u32) -> Self {
        Self(self.0 ^ (1 << i))
    }

    pub fn invert(&self, alphabet: &String) -> Self {
        Self(!self.0 & ((1 << alphabet.len()) - 1))
    }

    /// Check if no character is set.
    pub fn empty(&self) -> bool {
        self.0 == 0
    }

    /// Bitwise OR: Get characters that appear in any of the operands.
    pub fn merge(&self, other: &Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Bitwise AND: Get characters that appear in both operands at the same time.
    pub fn intersect(&self, other: &Self) -> Self {
        Self(self.0 & other.0)
    }

    pub fn has_overlap(&self, other: &Self) -> bool {
        !self.intersect(other).empty()
    }

    /// Bitwise XOR: Get characters that appear in one and only-one of the operands.
    pub fn flip(&self, other: &Self) -> Self {
        Self(self.0 ^ other.0)
    }

    pub fn contains(&self, i: u32) -> bool {
        self.0 & 1 << i != 0
    }

    /// Number of characters in this word.
    pub fn count(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn bits(&self) -> impl Iterator<Item = u32> {
        let mut tmp = self.0;
        std::iter::from_fn(move || match tmp {
            0 => None,
            _ => {
                let index = tmp.trailing_zeros();
                tmp ^= 1 << index;
                Some(index)
            }
        })
    }
}
