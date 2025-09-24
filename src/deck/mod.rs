//! Defines the standard deck of cards and masks for working with unordered sets of them
//!
//! This uses packed representations and bitmasks to efficiently represent sets of cards, which is useful for many purposes such as hand evaluation, simulation, and game state representation.
//! 
//! TODO: document this better, and write it up
//! 

use rand::Rng;

use core::fmt;
use std::fmt::{Debug, Display};
use std::str::FromStr;
use std::ops::{BitOr, BitAnd, BitXor, Not};

use serde::{Serialize, Deserialize};

// a helper that counts the number of items in a sequence of macro arguments
macro_rules! count_items {
    () => { 0 };
    ($head:ident $(, $tail:ident)*) => { 1 + count_items!($($tail),*) };
}

// a macro that defines a 'kind', which is a packed enumeration of items with associated data per each
macro_rules! make_kind {
    (
        $kind:ident($type:ident) : $repr:ty {
            $( $name:ident => $data:expr ),* $(,)?
        }
    ) => {
        #[repr($repr)]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
        pub enum $kind {
            $( $name , )*
        }

        impl $kind {
            pub const NUM: usize = count_items!($( $name ),*);
            pub const ALL: [Self; Self::NUM] = [
                $( Self::$name , )*
            ];

            pub const DATA: &'static [$type; Self::NUM] = &[
                $( $data, )*
            ];

            pub const fn index(self) -> $repr {
                self as $repr
            }

            pub const fn from_index(index: $repr) -> Self {
                Self::ALL[index as usize]
            }

            pub const fn data(self) -> &'static $type {
                &Self::DATA[self.index() as usize]
            }
        }
    };
}

macro_rules! make_kind_prod {
    (
        $kind:ident : $repr:ty = ($lname:ident: $lkind:ty) * ($rname:ident: $rkind:ty)
    ) => {
        // packed combination of two kinds into a single u8
        // #[repr($repr)]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
        pub struct $kind($repr);

        impl $kind {
            pub const NUM: usize = <$lkind>::NUM * <$rkind>::NUM;
            pub const ALL: [Self; Self::NUM] = {
                let mut all = [Self(0); Self::NUM];
                let mut i = 0;
                while i < Self::NUM {
                    all[i] = Self::from_index(i as $repr);
                    i += 1;
                }
                all
            };

            pub const fn new($lname: $lkind, $rname: $rkind) -> Self {
                Self::from_index($lname.index() + $rname.index() * <$lkind>::NUM as $repr)

                // other layout (transposed bit pattern)
                // Self::from_index($rname.index() + $lname.index() * <$rkind>::NUM as $repr)
            }

            pub const fn from_index(index: $repr) -> Self {
                Self(index as $repr)
            }

            pub const fn index(self) -> $repr {
                self.0 as $repr
            }
            
            pub const fn $lname(self) -> $lkind {
                <$lkind>::ALL[(self.0 as usize % <$lkind>::NUM) as usize]

                // other layout (transposed bit pattern)
                // <$lkind>::ALL[(self.0 as usize / <$rkind>::NUM) as usize]
            }

            pub const fn $rname(self) -> $rkind {
                <$rkind>::ALL[(self.0 as usize / <$lkind>::NUM) as usize]

                // other layout (transposed bit pattern)
                // <$rkind>::ALL[(self.0 as usize % <$rkind>::NUM) as usize]
            }
        }
    };
}

macro_rules! make_mask {
    (
        $(#[$attr:meta])* $mask:ident : $repr:ty = { $name:ident: $kind:ty }
    ) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
        $(#[$attr])*
        pub struct $mask($repr);

        impl $mask {
            pub const NONE: Self = Self::new(0);
            pub const FULL: Self = Self::new(((1 as $repr) << <$kind>::NUM) - 1);

            pub const fn new(bits: $repr) -> Self {
                Self(bits)
            }

            pub const fn bits(self) -> $repr {
                self.0
            }

            pub fn from_many(many: &[$kind]) -> Self {
                let mut mask = Self::new(Self::NONE.bits());
                let mut i = 0;
                while i < many.len() {
                    mask = mask | Self::from(many[i]);
                    i += 1;
                }
                mask
            }

            pub const fn empty(&self) -> bool {
                self.bits() == 0
            }

            pub const fn count(&self) -> usize {
                self.bits().count_ones() as usize
            }

            pub const fn contains(&self, other: Self) -> bool {
                (self.bits() & other.bits()) == other.bits()
            }

            pub const fn inverse(&self) -> Self {
                Self::new(Self::FULL.bits() ^ self.bits())
            }

            pub fn iter(&self) -> impl Iterator<Item = $kind> {
                <$kind>::ALL.iter().map(|k| *k).filter(|&k| self.contains(Self::from(k)))
            }

            pub fn iter_reverse(&self) -> impl Iterator<Item = $kind> {
                <$kind>::ALL.iter().rev().map(|k| *k).filter(|&k| self.contains(Self::from(k)))
            }

            pub fn topn<const N: usize>(&self) -> Self {
                let mut out = Self::NONE;
                let mut i = 0;
                for kind in self.iter_reverse() {
                    out = out | Self::from(kind);
                    i += 1;
                    if i >= N {
                        break;
                    }
                }
                out
            }

            pub fn top1(&self) -> Self { self.topn::<1>() }
            pub fn top2(&self) -> Self { self.topn::<2>() }
            pub fn top3(&self) -> Self { self.topn::<3>() }
            pub fn top4(&self) -> Self { self.topn::<4>() }
            pub fn top5(&self) -> Self { self.topn::<5>() }

            pub fn top(&self) -> Option<$kind> { self.topn::<1>().iter().next() }

        }

        impl BitOr for $mask {
            type Output = Self;
            fn bitor(self, other: Self) -> Self {
                Self::new(self.bits() | other.bits())
            }
        }

        impl BitAnd for $mask {
            type Output = Self;
            fn bitand(self, other: Self) -> Self {
                Self::new(self.bits() & other.bits())
            }
        }

        impl BitXor for $mask {
            type Output = Self;
            fn bitxor(self, other: Self) -> Self {
                Self::new(self.bits() ^ other.bits())
            }
        }

        // inverse
        impl Not for $mask {
            type Output = Self;
            fn not(self) -> Self {
                self.inverse()
            }
        }

        impl From<$kind> for $mask {
            fn from($name: $kind) -> Self {
                Self::new((1 as $repr) << $name.index())
            }
        }

        impl Display for $mask {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                // to display, just emit all the cards in the mask as a string
                for card in self.iter_reverse() {
                    write!(f, "{}", card)?;
                }
                Ok(())
            }
        }

        impl Debug for $mask {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}::from(\"", stringify!($mask))?;
                for (i, card) in self.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", card)?;
                }
                write!(f, "\")")?;
                Ok(())
            }
        }

        impl FromStr for $mask {
            type Err = &'static str;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let mut mask = Self::NONE;

                if s.contains(',') {
                    // if there is a comma, then we need to parse each card separately
                    for card in s.split(',') {
                        mask = mask | Self::from(<$kind>::from_str(card)?);
                    }
                } else {
                    // iterate over N chars each
                    // TODO: clean this up, allow comma separation
                    let expected_len = Self::from(<$kind>::from_index(0)).to_string().len();
                    
                    for i in (0..s.len()).step_by(expected_len) {
                        let card = <$kind>::from_str(&s[i..i+expected_len])?;
                        mask = mask | Self::from(card);
                    }
                }
                Ok(mask)
            }
        }

        impl From<&str> for $mask {
            fn from(s: &str) -> Self {
                Self::from_str(s).unwrap()
            }
        }
    }
}

/// Per-rank data
pub struct RankData {
    pub text: &'static str,
}

make_kind! {
    Rank(RankData) : u8 {
        Two      => RankData { text: "2" },
        Three    => RankData { text: "3" },
        Four     => RankData { text: "4" },
        Five     => RankData { text: "5" },
        Six      => RankData { text: "6" },
        Seven    => RankData { text: "7" },
        Eight    => RankData { text: "8" },
        Nine     => RankData { text: "9" },
        Ten      => RankData { text: "T" },
        Jack     => RankData { text: "J" },
        Queen    => RankData { text: "Q" },
        King     => RankData { text: "K" },
        Ace      => RankData { text: "A" },
    }
}

impl Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.data().text)
    }
}

impl FromStr for Rank {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for rank in Rank::ALL {
            if rank.data().text == s {
                return Ok(rank);
            }
        }
        Err("invalid rank")
    }
}

impl From<&str> for Rank {
    fn from(s: &str) -> Self {
        Self::from_str(s).unwrap()
    }
}


/// Per-suit data
pub struct SuitData {
    pub text: &'static str,
}

make_kind! {
    Suit(SuitData) : u8 {
        Spades   => SuitData { text: "s" },
        Hearts   => SuitData { text: "h" },
        Clubs    => SuitData { text: "c" },
        Diamonds => SuitData { text: "d" },
    }
}

impl Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.data().text)
    }
}

impl FromStr for Suit {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for suit in Suit::ALL {
            if suit.data().text == s {
                return Ok(suit);
            }
        }
        Err("invalid suit")
    }
}

impl From<&str> for Suit {
    fn from(s: &str) -> Self {
        Self::from_str(s).unwrap()
    }
}


make_kind_prod! {
    Card : u8 = (rank: Rank) * (suit: Suit)
}

impl Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.rank(), self.suit())
    }
}

impl FromStr for Card {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 2 {
            Ok(Self::new(Rank::from_str(&s[..1])?, Suit::from_str(&s[1..])?))
        } else {
            Err("invalid card")
        }
    }
}

impl From<&str> for Card {
    fn from(s: &str) -> Self {
        Self::from_str(s).unwrap()
    }
}


make_mask! {
    RankMask : u16 = { rank: Rank }
}

make_mask! {
    SuitMask : u8 = { suit: Suit }
}

make_mask! {
    CardMask : u64 = { card: Card }
}


impl CardMask {

    pub fn unsuited(self) -> RankMask {
        let mut mask = RankMask::NONE;
        for card in self.iter() {
            mask = mask | RankMask::from(card.rank());
        }
        mask
    }

    /// Calculate the ranks present for a given suit in the cards
    pub fn of_suit(self, suit: Suit) -> RankMask {
        let bits = self.bits() >> (13 * suit.index());
        RankMask::new(bits as u16) & RankMask::FULL
    }

    /// Calculate the suits present for a given rank in the cards
    pub fn of_rank(self, rank: Rank) -> SuitMask {
        let hackbits = 1u64 << rank.index();
        let mut mask = SuitMask::NONE;
        if (self.bits() & hackbits) != 0 {
            mask = mask | Suit::Spades.into();
        }
        if (self.bits() & (hackbits << 13)) != 0 {
            mask = mask | Suit::Hearts.into();
        }
        if (self.bits() & (hackbits << 26)) != 0 {
            mask = mask | Suit::Clubs.into();
        }
        if (self.bits() & (hackbits << 39)) != 0 {
            mask = mask | Suit::Diamonds.into();
        }
        mask
    }

    /// Calculate the number of cards of a given rank present in the cards
    pub fn of_rank_count(self, rank: Rank) -> usize {
        // convert the rank to a bitmask
        let rank_bits = 1u64 << rank.index();

        // get the bitmask corresponding to all suits for the given rank
        let rank_all_suits = rank_bits | (rank_bits << 13) | (rank_bits << 26) | (rank_bits << 39);

        // now, count the number of bits set in the bitmask
        (rank_all_suits & self.bits()).count_ones() as usize
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_roundtrip_idx() {
        for idx in 0..Rank::NUM {
            let rank = Rank::from_index(idx as u8);
            assert_eq!(rank, Rank::ALL[idx], "created rank does not match expected builtin rank from lookup table");
            assert_eq!(idx, rank.index() as usize, "creation index does not match computed index");
        }
    }

    #[test]
    fn rank_roundtrip_all() {
        for rank in Rank::ALL {
            let idx = rank.index() as usize;
            assert_eq!(rank, Rank::ALL[idx], "created rank does not match expected builtin rank from lookup table");
            assert_eq!(idx, rank.index() as usize, "creation index does not match computed index");
        }
    }

    #[test]
    fn rank_parse_all() {
        for rank in Rank::ALL {
            let text = format!("{}", rank);
            assert_eq!(rank, Rank::from_str(&text).unwrap(), "created rank does not match expected builtin rank from lookup table");
            assert_eq!(text, rank.data().text, "created rank text does not match expected builtin rank text");
        }
    }

    #[test]
    fn rank_parse_invalid() {
        assert!(Rank::from_str("invalid").is_err());
        assert!(Rank::from_str("22").is_err());
        assert!(Rank::from_str("23").is_err());
        assert!(Rank::from_str("1").is_err());
        assert!(Rank::from_str("10").is_err());
        assert!(Rank::from_str("B").is_err());
        assert!(Rank::from_str("C").is_err());
        assert!(Rank::from_str("AK").is_err());
    }

    #[test]
    fn suit_roundtrip_idx() {
        for idx in 0..Suit::NUM {
            let suit = Suit::from_index(idx as u8);
            assert_eq!(suit, Suit::ALL[idx], "created suit does not match expected builtin suit from lookup table");
            assert_eq!(idx, suit.index() as usize, "creation index does not match computed index");
        }
    }

    #[test]
    fn suit_roundtrip_all() {
        for suit in Suit::ALL {
            let idx = suit.index() as usize;
            assert_eq!(suit, Suit::ALL[idx], "created suit does not match expected builtin suit from lookup table");
            assert_eq!(idx, suit.index() as usize, "creation index does not match computed index");
        }
    }

    #[test]
    fn suit_parse_all() {
        for suit in Suit::ALL {
            let text = format!("{}", suit);
            assert_eq!(suit, Suit::from_str(&text).unwrap(), "created suit does not match expected builtin suit from lookup table");
            assert_eq!(text, suit.data().text, "created suit text does not match expected builtin suit text");
        }
    }

    #[test]
    fn suit_parse_invalid() {
        assert!(Suit::from_str("invalid").is_err());
        assert!(Suit::from_str("A").is_err());
        assert!(Suit::from_str("AB").is_err());
        assert!(Suit::from_str("AH").is_err());
        assert!(Suit::from_str("AC").is_err());
        assert!(Suit::from_str("AD").is_err());
    }

    #[test]
    fn card_roundtrip_idx() {
        for idx in 0..Card::NUM {
            let card = Card::from_index(idx as u8);
            assert_eq!(card, Card::ALL[idx], "created card does not match expected builtin card from lookup table");
            assert_eq!(idx, card.index() as usize, "creation index does not match computed index");
        }
    }

    #[test]
    fn card_roundtrip_all() {
        for card in Card::ALL {
            let idx = card.index() as usize;
            assert_eq!(card, Card::ALL[idx], "created card does not match expected builtin card from lookup table");
            assert_eq!(idx, card.index() as usize, "creation index does not match computed index");
        }
    }

    #[test]
    fn card_parse_all() {
        for card in Card::ALL {
            let text = format!("{}", card);
            assert_eq!(card, Card::from_str(&text).unwrap(), "created card does not match expected builtin card from lookup table");
            assert_eq!(text, card.to_string(), "created card text does not match expected builtin card text");
        }
    }

    #[test]
    fn card_parse_invalid() {
        assert!(Card::from_str("invalid").is_err());
        assert!(Card::from_str("23").is_err());
        assert!(Card::from_str("23").is_err());
        assert!(Card::from_str("23").is_err());
    }

    #[test]
    fn card_new() {
        // keep special cases here in case something change as a canary test
        assert_eq!(Card::new(Rank::Two, Suit::Spades), Card::from_str("2s").unwrap());
        assert_eq!(Card::new(Rank::Three, Suit::Spades), Card::from_str("3s").unwrap());
        assert_eq!(Card::new(Rank::Four, Suit::Spades), Card::from_str("4s").unwrap());
        assert_eq!(Card::new(Rank::Five, Suit::Spades), Card::from_str("5s").unwrap());
        assert_eq!(Card::new(Rank::Six, Suit::Spades), Card::from_str("6s").unwrap());
        assert_eq!(Card::new(Rank::Seven, Suit::Spades), Card::from_str("7s").unwrap());
        assert_eq!(Card::new(Rank::Eight, Suit::Spades), Card::from_str("8s").unwrap());
        assert_eq!(Card::new(Rank::Nine, Suit::Spades), Card::from_str("9s").unwrap());
        assert_eq!(Card::new(Rank::Ten, Suit::Spades), Card::from_str("Ts").unwrap());
        assert_eq!(Card::new(Rank::Jack, Suit::Spades), Card::from_str("Js").unwrap());
        assert_eq!(Card::new(Rank::Queen, Suit::Spades), Card::from_str("Qs").unwrap());
        assert_eq!(Card::new(Rank::King, Suit::Spades), Card::from_str("Ks").unwrap());
        assert_eq!(Card::new(Rank::Ace, Suit::Spades), Card::from_str("As").unwrap());
        assert_eq!(Card::new(Rank::Two, Suit::Hearts), Card::from_str("2h").unwrap());
        assert_eq!(Card::new(Rank::Three, Suit::Hearts), Card::from_str("3h").unwrap());
        assert_eq!(Card::new(Rank::Four, Suit::Hearts), Card::from_str("4h").unwrap());
        assert_eq!(Card::new(Rank::Five, Suit::Hearts), Card::from_str("5h").unwrap());
        assert_eq!(Card::new(Rank::Six, Suit::Hearts), Card::from_str("6h").unwrap());
        assert_eq!(Card::new(Rank::Seven, Suit::Hearts), Card::from_str("7h").unwrap());
        assert_eq!(Card::new(Rank::Eight, Suit::Hearts), Card::from_str("8h").unwrap());
        assert_eq!(Card::new(Rank::Nine, Suit::Hearts), Card::from_str("9h").unwrap());
        assert_eq!(Card::new(Rank::Ten, Suit::Hearts), Card::from_str("Th").unwrap());
        assert_eq!(Card::new(Rank::Ten, Suit::Hearts), Card::from_str("Th").unwrap());
        assert_eq!(Card::new(Rank::Queen, Suit::Hearts), Card::from_str("Qh").unwrap());
        assert_eq!(Card::new(Rank::King, Suit::Hearts), Card::from_str("Kh").unwrap());
        assert_eq!(Card::new(Rank::Ace, Suit::Hearts), Card::from_str("Ah").unwrap());
        assert_eq!(Card::new(Rank::Two, Suit::Clubs), Card::from_str("2c").unwrap());
        assert_eq!(Card::new(Rank::Three, Suit::Clubs), Card::from_str("3c").unwrap());
        assert_eq!(Card::new(Rank::Four, Suit::Clubs), Card::from_str("4c").unwrap());
        assert_eq!(Card::new(Rank::Five, Suit::Clubs), Card::from_str("5c").unwrap());
        assert_eq!(Card::new(Rank::Six, Suit::Clubs), Card::from_str("6c").unwrap());
        assert_eq!(Card::new(Rank::Seven, Suit::Clubs), Card::from_str("7c").unwrap());
        assert_eq!(Card::new(Rank::Eight, Suit::Clubs), Card::from_str("8c").unwrap());
        assert_eq!(Card::new(Rank::Nine, Suit::Clubs), Card::from_str("9c").unwrap());
        assert_eq!(Card::new(Rank::Ten, Suit::Clubs), Card::from_str("Tc").unwrap());
        assert_eq!(Card::new(Rank::Jack, Suit::Clubs), Card::from_str("Jc").unwrap());
        assert_eq!(Card::new(Rank::Queen, Suit::Clubs), Card::from_str("Qc").unwrap());
        assert_eq!(Card::new(Rank::King, Suit::Clubs), Card::from_str("Kc").unwrap());
        assert_eq!(Card::new(Rank::Ace, Suit::Clubs), Card::from_str("Ac").unwrap());
        assert_eq!(Card::new(Rank::Two, Suit::Diamonds), Card::from_str("2d").unwrap());
        assert_eq!(Card::new(Rank::Three, Suit::Diamonds), Card::from_str("3d").unwrap());
        assert_eq!(Card::new(Rank::Four, Suit::Diamonds), Card::from_str("4d").unwrap());
        assert_eq!(Card::new(Rank::Five, Suit::Diamonds), Card::from_str("5d").unwrap());
        assert_eq!(Card::new(Rank::Six, Suit::Diamonds), Card::from_str("6d").unwrap());
        assert_eq!(Card::new(Rank::Seven, Suit::Diamonds), Card::from_str("7d").unwrap());
        assert_eq!(Card::new(Rank::Eight, Suit::Diamonds), Card::from_str("8d").unwrap());
        assert_eq!(Card::new(Rank::Nine, Suit::Diamonds), Card::from_str("9d").unwrap());
        assert_eq!(Card::new(Rank::Ten, Suit::Diamonds), Card::from_str("Td").unwrap());
        assert_eq!(Card::new(Rank::Jack, Suit::Diamonds), Card::from_str("Jd").unwrap());
        assert_eq!(Card::new(Rank::Queen, Suit::Diamonds), Card::from_str("Qd").unwrap());
        assert_eq!(Card::new(Rank::King, Suit::Diamonds), Card::from_str("Kd").unwrap());
        assert_eq!(Card::new(Rank::Ace, Suit::Diamonds), Card::from_str("Ad").unwrap());
    }

    // #[test]
    // fn card_idx() {
    //     // keep special cases here in case something change as a canary test
    //     assert_eq!(Card::from_index(0), Card::from_str("2s").unwrap());
    //     assert_eq!(Card::from_index(1), Card::from_str("3s").unwrap());
    //     assert_eq!(Card::from_index(2), Card::from_str("4s").unwrap());
    //     assert_eq!(Card::from_index(3), Card::from_str("5s").unwrap());
    //     assert_eq!(Card::from_index(4), Card::from_str("6s").unwrap());
    //     assert_eq!(Card::from_index(5), Card::from_str("7s").unwrap());
    //     assert_eq!(Card::from_index(6), Card::from_str("8s").unwrap());
    //     assert_eq!(Card::from_index(7), Card::from_str("9s").unwrap());
    //     assert_eq!(Card::from_index(8), Card::from_str("Ts").unwrap());
    //     assert_eq!(Card::from_index(9), Card::from_str("Js").unwrap());
    //     assert_eq!(Card::from_index(10), Card::from_str("Qs").unwrap());
    //     assert_eq!(Card::from_index(11), Card::from_str("Ks").unwrap());
    //     assert_eq!(Card::from_index(12), Card::from_str("As").unwrap());
    //     assert_eq!(Card::from_index(13), Card::from_str("2h").unwrap());
    //     assert_eq!(Card::from_index(14), Card::from_str("3h").unwrap());
    //     assert_eq!(Card::from_index(15), Card::from_str("4h").unwrap());
    //     assert_eq!(Card::from_index(16), Card::from_str("5h").unwrap());
    //     assert_eq!(Card::from_index(17), Card::from_str("6h").unwrap());
    //     assert_eq!(Card::from_index(18), Card::from_str("7h").unwrap());
    //     assert_eq!(Card::from_index(19), Card::from_str("8h").unwrap());
    //     assert_eq!(Card::from_index(20), Card::from_str("9h").unwrap());
    //     assert_eq!(Card::from_index(21), Card::from_str("Th").unwrap());
    //     assert_eq!(Card::from_index(22), Card::from_str("Jh").unwrap());
    //     assert_eq!(Card::from_index(23), Card::from_str("Qh").unwrap());
    //     assert_eq!(Card::from_index(24), Card::from_str("Kh").unwrap());
    //     assert_eq!(Card::from_index(25), Card::from_str("Ah").unwrap());
    //     assert_eq!(Card::from_index(26), Card::from_str("2c").unwrap());
    //     assert_eq!(Card::from_index(27), Card::from_str("3c").unwrap());
    //     assert_eq!(Card::from_index(28), Card::from_str("4c").unwrap());
    //     assert_eq!(Card::from_index(29), Card::from_str("5c").unwrap());
    //     assert_eq!(Card::from_index(30), Card::from_str("6c").unwrap());
    //     assert_eq!(Card::from_index(31), Card::from_str("7c").unwrap());
    //     assert_eq!(Card::from_index(32), Card::from_str("8c").unwrap());
    //     assert_eq!(Card::from_index(33), Card::from_str("9c").unwrap());
    //     assert_eq!(Card::from_index(34), Card::from_str("Tc").unwrap());
    //     assert_eq!(Card::from_index(35), Card::from_str("Jc").unwrap());
    //     assert_eq!(Card::from_index(36), Card::from_str("Qc").unwrap());
    //     assert_eq!(Card::from_index(37), Card::from_str("Kc").unwrap());
    //     assert_eq!(Card::from_index(38), Card::from_str("Ac").unwrap());
    //     assert_eq!(Card::from_index(39), Card::from_str("2d").unwrap());
    //     assert_eq!(Card::from_index(40), Card::from_str("3d").unwrap());
    //     assert_eq!(Card::from_index(41), Card::from_str("4d").unwrap());
    //     assert_eq!(Card::from_index(42), Card::from_str("5d").unwrap());
    //     assert_eq!(Card::from_index(43), Card::from_str("6d").unwrap());
    //     assert_eq!(Card::from_index(44), Card::from_str("7d").unwrap());
    //     assert_eq!(Card::from_index(45), Card::from_str("8d").unwrap());
    //     assert_eq!(Card::from_index(46), Card::from_str("9d").unwrap());
    //     assert_eq!(Card::from_index(47), Card::from_str("Td").unwrap());
    //     assert_eq!(Card::from_index(48), Card::from_str("Jd").unwrap());
    //     assert_eq!(Card::from_index(49), Card::from_str("Qd").unwrap());
    //     assert_eq!(Card::from_index(50), Card::from_str("Kd").unwrap());
    //     assert_eq!(Card::from_index(51), Card::from_str("Ad").unwrap());
    // }


    #[test]
    pub fn masks_empty() {
        assert_eq!(RankMask::NONE, "".into());
        assert_eq!(SuitMask::NONE, "".into());
        assert_eq!(CardMask::NONE, "".into());
    }

    #[test]
    pub fn cardmask_each() {
        for card in Card::ALL {
            let m: CardMask = card.into();
            assert_eq!(m.count(), 1, "expecteded exactly 1 entry in the mask");
            assert!(m.contains(card.into()), "expected the mask to contain the card");
            assert!(m.iter().next().unwrap() == card, "expected the mask to contain the card");
            assert!(m.iter().collect::<Vec<_>>() == vec![card], "expected the mask to contain the card");
            assert!(m.bits() == 1 << card.index(), "expected the mask to have the correct bits");
        }
    }

    #[test]
    pub fn cardmask_x2() {
        for c1 in Card::ALL {
            for c2 in Card::ALL {
                let m1 = CardMask::from(c1);
                let m2 = CardMask::from(c2);
                if m1 > m2 {
                    break;
                }
                let m12 = m1 | m2;
                if c1 == c2 {
                    assert_eq!(m12.count(), 1, "expected exactly 1 entry in the mask");
                    assert!(m12.contains(c1.into()), "expected the mask to contain the card");
                    assert!(m12.iter().next().unwrap() == c1, "expected the mask to contain the card");
                    assert!(m12.iter().collect::<Vec<_>>() == vec![c1], "expected the mask to contain the card");
                    assert!(m12.bits() == 1 << c1.index(), "expected the mask to have the correct bits");
                } else {
                    assert_eq!(m12.count(), 2, "expected exactly 2 entries in the mask");
                    assert!(m12.contains(c1.into()), "expected the mask to contain the card");
                    assert!(m12.contains(c2.into()), "expected the mask to contain the card");
                    assert!(m12.iter().next().unwrap() == c1, "expected the mask to contain the card");
                    assert!(m12.iter().collect::<Vec<_>>() == vec![c1, c2], "expected the mask to contain the card");
                    assert!(m12.bits() == (1 << c1.index()) | (1 << c2.index()), "expected the mask to have the correct bits");
                }
            }
        }
    }

}


/// Randomly samples `num` cards from `src`, a set of available cards (with order, without replacement)
pub fn sample_cards_ordered<R: Rng>(src: CardMask, num: usize, rng: &mut R) -> Vec<Card> {
    assert!(src.count() >= num);
    let mut res: Vec<Card> = Vec::new();
    while res.len() < num {
        // generate a random card index
        let card_idx = rng.random_range(0..Card::NUM);

        // turn it into a proper card
        let card = Card::from_index(card_idx as u8);

        let card_mask = CardMask::from(card);

        // check if the card is in the source mask (i.e. is available)
        if src.contains(card_mask) && !res.contains(&card) {
            // only if it is available, add it to the result
            res.push(card);
        }
    }

    assert_eq!(res.len(), num);
    res
}

/// Randomly samples `num` cards from `src`, a set of available cards (unordered, without replacement)
pub fn sample_cards<R: Rng>(src: CardMask, num: usize, rng: &mut R) -> CardMask {
    assert!(src.count() >= num);

    // the result, which starts empty
    let mut res = CardMask::NONE;

    // TODO: smarter method that only calls the RNG exactly `n` times? (or 1?)
    // keep going until we have `n` cards
    while res.count() < num {
        // generate a random card index 
        let card_idx = rng.random_range(0..Card::NUM);

        // turn it into a proper card
        let card = Card::from_index(card_idx as u8);
        
        // add it to the result, which may do nothing if the card is not in the source mask
        // if so, it won't change the count and we'll keep going until we have `n` cards
        res = res | (CardMask::from(card) & src);
    }
    res
}
