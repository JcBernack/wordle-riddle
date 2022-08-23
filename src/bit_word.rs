use std::fmt::Debug;

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BitWord(u32);

impl BitWord {
    pub fn new(word: &str) -> Self {
        word.chars().fold(Self::default(), |word, c| word.toggle(c))
    }

    fn encode(c: char) -> Self {
        Self(1 << Self::char_pos(c))
    }

    fn char_pos(c: char) -> u32 {
        c.to_ascii_uppercase() as u32 - 'A' as u32
    }

    pub fn toggle(&self, c: char) -> Self {
        Self(self.0 ^ Self::encode(c).0)
    }

    pub fn invert(&self) -> Self {
        Self(!self.0 & ((1 << 26) - 1))
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

    // Bitwise XOR: Get characters that appear in one and only-one of the operands.
    pub fn flip(&self, other: &Self) -> Self {
        Self(self.0 ^ other.0)
    }

    pub fn contains(&self, c: char) -> bool {
        Self::encode(c).has_overlap(self)
    }

    pub fn count(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn chars(&self) -> impl Iterator<Item = char> {
        let mut x = self.0;
        std::iter::from_fn(move || {
            let zeros = x.trailing_zeros();
            (zeros < u32::BITS).then(|| {
                char::from_u32({
                    x ^= 1 << zeros;
                    ('A' as u32) + zeros
                })
                .unwrap()
            })
        })
    }
}

impl Debug for BitWord {
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

// impl std::ops::BitOr for BitWord {
//     type Output = Self;
//
//     fn bitor(self, rhs: Self) -> Self::Output {
//         Self(self.0 | rhs.0)
//     }
// }
//
// impl std::ops::BitAnd for BitWord {
//     type Output = Self;
//
//     fn bitand(self, rhs: Self) -> Self::Output {
//         Self(self.0 & rhs.0)
//     }
// }
//
// impl std::ops::BitXor for BitWord {
//     type Output = Self;
//
//     fn bitxor(self, rhs: Self) -> Self::Output {
//         Self(self.0 ^ rhs.0)
//     }
// }
//
// impl std::ops::Not for BitWord {
//     type Output = Self;
//
//     fn not(self) -> Self::Output {
//         Self(!self.0 & ((1 << 26) - 1))
//     }
// }
