//! Bitmask utilities for unordered sets of cards, generally faster and more memory efficient than other methods

use serde::{Serialize, Deserialize};
use std::fmt;
use std::hash::Hash;
use std::str::FromStr;

use crate::deck::{Card, Rank, Suit};

macro_rules! make_mask {
    ( $(#[$attr:meta])* $name:ident = { $kind:ident } ) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
        $(#[$attr])* pub struct $name {
            bits: u64,
        }

        impl $name {
            pub const MASK_NONE: u64 = 0;
            pub const MASK_FULL: u64 = (1u64 << $kind::NUM) - 1;

            pub const fn none() -> Self {
                Self::from_bits(Self::MASK_NONE)
            }
            
            pub const fn full() -> Self {
                Self::from_bits(Self::MASK_FULL)
            }

            pub const fn from_bits(bits: u64) -> Self {
                Self { bits: bits }
            }

            pub const fn to_bits(self) -> u64 {
                self.bits
            }

            pub const fn from_single(kind: $kind) -> Self {
                Self::from_bits(1 << kind.to_index())
            }

            // implement with an into slice
            pub const fn from_multi(kinds: &[$kind]) -> Self {
                let mut mask = Self::none();
                let mut i = 0;
                while i < kinds.len() {
                    mask = mask.union(Self::from_single(kinds[i]));
                    i += 1;
                }
                mask
            }

            pub const fn union(&self, mask: Self) -> Self {
                Self::from_bits(self.bits | mask.bits)
            }
        
            pub const fn intersect(&self, mask: Self) -> Self {
                Self::from_bits(self.bits & mask.bits)
            }
            
            pub const fn difference(&self, mask: Self) -> Self {
                Self::from_bits(self.bits ^ mask.bits)
            }
        
            pub const fn empty(&self) -> bool {
                self.bits == 0
            }
        
            pub const fn count(&self) -> usize {
                self.bits.count_ones() as usize
            }

            pub const fn inverse(&self) -> Self {
                Self::from_bits(Self::MASK_FULL ^ self.bits)
            }

            pub fn contains(&self, kind: $kind) -> bool {
                (self.bits & (1 << kind.to_index())) != 0
            }

            pub fn topn<const N: usize>(&self) -> Self {
                let mut out = Self::none();
                let mut i = 0;
                for kind in self.iter_reverse() {
                    out = out.union(Self::from_single(kind));
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


            pub fn iter(&self) -> impl Iterator<Item = $kind> {
                // TODO: make this const
                (0..$kind::NUM).filter_map(|i| if (self.bits & (1 << i)) != 0 {
                    Some($kind::from_index(i as u8))
                } else {
                    None
                })
            }

            pub fn iter_reverse(&self) -> impl Iterator<Item = $kind> {
                (0..$kind::NUM).rev().filter_map(|i| if (self.bits & (1 << i)) != 0 {
                    Some($kind::from_index(i as u8))
                } else {
                    None
                })
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                // to display, just emit all the cards in the mask as a string
                for card in self.iter_reverse() {
                    write!(f, "{}", card)?;
                }
                Ok(())
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                // format as binary string with exactly 'NUM' bits, but zero-padded
                // fmt::Display::fmt(self, f)

                write!(f, "{}::from(\"", stringify!($name))?;
                for (i, card) in self.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", card)?;
                }
                write!(f, "\")")?;
                Ok(())

                // let extra_prefix_len = 2;
                // write!(f, "{}::from_bits({:#0width$b})", stringify!($name), self.bits, width = $kind::NUM + extra_prefix_len)?;
                // Ok(())
            }
        }

        impl FromStr for $name {
            type Err = &'static str;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                // continually parse the base strings
                let mut mask = Self::none();
                // iterate over N chars each
                // TODO: clean this up, allow comma separation
                let expected_len = Self::from_single($kind::from_index(0)).to_string().len();
                for i in (0..s.len()).step_by(expected_len) {
                    let card = $kind::from_str(&s[i..i+expected_len])?;
                    mask = mask.union(Self::from_single(card));
                }
                Ok(mask)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self::from_str(s).unwrap()
            }
        }
    };
}

make_mask! {
    /// A mask for unordered sets of cards, which can contain anywhere between 0 and 52 cards
    CardMask = { Card }
}

impl CardMask {
    /// Construct a rank mask from a card mask by removing the suit information from cards
    pub fn unsuited(self) -> RankMask {
        let mut mask = RankMask::none();
        // TODO: faster way by just shifting the bits and unioning them? since they are lower-order bits
        for card in self.iter() {
            mask = mask.union(RankMask::from_single(card.rank()));
        }
        mask
    }

    /// Construct a suit mask from a card mask by removing the rank information from cards
    pub fn unranked(self) -> SuitMask {
        let mut mask = SuitMask::none();
        for card in self.iter() {
            mask = mask.union(SuitMask::from_single(card.suit()));
        }
        mask
    }

    pub fn from_suit(suit: Suit) -> Self {
        let mut mask = Self::none();
        for rank in Rank::ALL.iter() {
            mask = mask.union(Self::from_single(Card::new(*rank, suit)));
        }
        mask
    }

    pub fn from_rank(rank: Rank) -> Self {
        let mut mask = Self::none();
        for suit in Suit::ALL {
            mask = mask.union(Self::from_single(Card::new(rank, suit)));
        }
        mask
    }

    pub fn from_ranks(ranks: &[Rank]) -> Self {
        let mut mask = Self::none();
        for rank in ranks {
            mask = mask.union(Self::from_rank(*rank));
        }
        mask
    }

    /// Iterate over the ranks per each suit from a card mask
    pub fn each_suit(&self) -> impl Iterator<Item = (Suit, RankMask)> {
        Suit::ALL.iter()
            .map(|suit| (*suit, self.intersect(Self::from_suit(*suit)).unsuited()))
    }

    /// Iterate over the ranks from a card mask
    pub fn each_rank(&self) -> impl Iterator<Item = (Rank, SuitMask)> {
        Rank::ALL.iter()
            .map(|rank| (*rank, self.intersect(Self::from_rank(*rank)).unranked()))
    }
}

make_mask! {
    RankMask = { Rank }
}

make_mask! {
    SuitMask = { Suit }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmp() {
        assert!(CardMask::from_str("2s3h4s5h7s") < CardMask::from_str("2s3h4s5h8s"));
        assert!(Card::from_str("2s") == Card::from_str("2s"));
        assert!(Card::from_str("2s") != Card::from_str("3h"));
        assert!(Card::from_str("2s") < Card::from_str("3h"));
    }


    #[test]
    fn test_cardmask_roundtrip_all() {
        for card in Card::ALL {
            let mask = CardMask::from_single(card);
            assert_eq!(mask.count(), 1);
            assert_eq!(mask.to_bits(), 1 << card.to_index());
            assert_eq!(card, mask.iter().next().unwrap());
            assert_eq!(card, mask.iter_reverse().next().unwrap());
            assert_eq!(card, mask.topn::<1>().iter().next().unwrap());
            assert_eq!(card, mask.topn::<1>().iter_reverse().next().unwrap());
            assert_eq!(card, mask.topn::<1>().iter().next().unwrap());
        }
    }

    #[test]
    fn test_cardmask_roundtrip_index() {
        for i in 0..Card::NUM as u8 {
            assert_eq!(CardMask::from_single(Card::from_index(i as u8)), CardMask::from_bits(1 << i));
        }
    }

    #[test]
    fn test_cardmask_x0() {
        let m = CardMask::none();
        assert_eq!(m.count(), 0);
        assert_eq!(m.to_bits(), 0);
        assert_eq!(m.empty(), true);
        assert_eq!(m.iter().count(), 0);
        assert_eq!(m.iter_reverse().count(), 0);
        assert_eq!(m.topn::<1>().iter().count(), 0);
        assert_eq!(m.topn::<1>().iter_reverse().count(), 0);
        assert_eq!(m.topn::<1>().iter().count(), 0);
        assert_eq!(m.iter().collect::<Vec<_>>(), &[]);
    }

    #[test]
    fn test_cardmask_x1() {
        for c in Card::ALL {
            let m = CardMask::from_single(c);
            assert_eq!(m.count(), 1);
            assert_eq!(m.to_bits(), 1 << c.to_index());
            assert_eq!(m.iter().count(), 1);
            assert_eq!(m.iter_reverse().count(), 1);
            assert_eq!(m.topn::<1>().iter().count(), 1);
            assert_eq!(m.topn::<1>().iter_reverse().count(), 1);
            assert_eq!(m.topn::<1>().iter().count(), 1);
            assert_eq!(m.iter().collect::<Vec<_>>(), &[c]);
        }
    }

    #[test]
    fn test_cardmask_x2() {
        for c1 in Card::ALL {
            for c2 in Card::ALL {
                let m1 = CardMask::from_single(c1);
                let m2 = CardMask::from_single(c2);
                let m12 = m1.union(m2);
                if c1 != c2 {
                    // unique, distinct cards, so they should both be present in the mask
                    assert_eq!(m12.count(), 2);
                    assert!(m12.contains(c1));
                    assert!(m12.contains(c2));
                    assert_eq!(m12.to_bits(), (1 << c1.to_index()) | (1 << c2.to_index()));
                    assert_eq!(m12.empty(), false);
                } else {
                    // same card
                    assert_eq!(m12.count(), 1);
                    assert!(m12.contains(c1));
                    assert_eq!(m12.to_bits(), 1 << c1.to_index());
                    assert_eq!(m12.empty(), false);
                    assert_eq!(m12.count(), 1);
                }
            }
        }
    }

    #[test]
    fn test_cardmask_x3() {
        for c1 in Card::ALL {
            for c2 in Card::ALL {
                for c3 in Card::ALL {
                    let m1 = CardMask::from_single(c1);
                    let m2 = CardMask::from_single(c2);
                    let m3 = CardMask::from_single(c3);
                    let m123 = m1.union(m2).union(m3);
                    let m132 = m1.union(m3).union(m2);
                    let m213 = m2.union(m1).union(m3);
                    let m231 = m2.union(m3).union(m1);
                    let m312 = m3.union(m1).union(m2);
                    let m321 = m3.union(m2).union(m1);

                    // all should be equal masks, order doesn't matter
                    assert_eq!(m123, m132);
                    assert_eq!(m123, m213);
                    assert_eq!(m123, m231);
                    assert_eq!(m123, m312);
                    assert_eq!(m123, m321);
                    assert_eq!(m123.to_bits(), m132.to_bits());
                    assert_eq!(m123.to_bits(), m213.to_bits());
                    assert_eq!(m123.to_bits(), m231.to_bits());
                    assert_eq!(m123.to_bits(), m312.to_bits());
                    assert_eq!(m123.to_bits(), m321.to_bits());

                    if c1 != c2 && c1 != c3 && c2 != c3 {
                        assert_eq!(m123.count(), 3);
                        assert!(m123.contains(c1));
                        assert!(m123.contains(c2));
                        assert!(m123.contains(c3));
                        assert_eq!(m123.to_bits(), (1 << c1.to_index()) | (1 << c2.to_index()) | (1 << c3.to_index()));
                        assert_eq!(m123.empty(), false);
                    }
                    if c1 != c3 && c1 != c2 && c3 != c2 {
                        assert_eq!(m132.count(), 3);
                        assert!(m132.contains(c1));
                        assert!(m132.contains(c3));
                        assert!(m132.contains(c2));
                        assert_eq!(m132.to_bits(), (1 << c1.to_index()) | (1 << c3.to_index()) | (1 << c2.to_index()));
                        assert_eq!(m132.empty(), false);
                    }
                    if c2 != c3 && c2 != c1 && c3 != c1 {
                        assert_eq!(m213.count(), 3);
                        assert!(m213.contains(c2));
                        assert!(m213.contains(c1));
                        assert!(m213.contains(c3));
                        assert_eq!(m213.to_bits(), (1 << c2.to_index()) | (1 << c1.to_index()) | (1 << c3.to_index()));
                        assert_eq!(m213.empty(), false);
                    }
                    if c2 != c3 && c2 != c1 && c3 != c1 {
                        assert_eq!(m231.count(), 3);
                        assert!(m231.contains(c2));
                        assert!(m231.contains(c3));
                        assert!(m231.contains(c1));
                        assert_eq!(m231.to_bits(), (1 << c2.to_index()) | (1 << c3.to_index()) | (1 << c1.to_index()));
                        assert_eq!(m231.empty(), false);
                    }
                    if c3 != c1 && c3 != c2 && c1 != c2 {
                        assert_eq!(m312.count(), 3);
                        assert!(m312.contains(c3));
                        assert!(m312.contains(c1));
                        assert!(m312.contains(c2));
                        assert_eq!(m312.to_bits(), (1 << c3.to_index()) | (1 << c1.to_index()) | (1 << c2.to_index()));
                        assert_eq!(m312.empty(), false);
                    }

                }
            }
        }
    }


    #[test]
    fn test_cardmask_suits() {
        for suit in Suit::ALL {
            let m = CardMask::from_suit(suit);
            assert_eq!(m.count(), Rank::NUM);
            // check specific binary structure, since other places assume it is contiguous (i.e. 13 bits per suit)
            assert_eq!(m.to_bits(), ((1 << Rank::NUM) - 1) << (suit.to_index() * Rank::NUM as u8));
        }
    }

    #[test]
    fn test_cardmask_unsuited() {
        for card in Card::ALL {
            let m = CardMask::from_single(card);
            let m_unsuited = m.unsuited();
            assert_eq!(m_unsuited.count(), 1);
        }
    }

    #[test]
    fn test_cardmask_each_suit() {
        for card in Card::ALL {
            let m = CardMask::from_single(card);
            assert_eq!(m.each_suit().count(), 4);
            assert_eq!(m.each_suit().filter(|(_, m)| m.contains(card.rank())).count(), 1);
            assert_eq!(m.each_suit().filter(|(_, m)| m.contains(card.rank())).next().unwrap().0, card.suit());
        }
    }
}


/*

// the base mask type, which can represent any set of cards
#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct CardMask {
    bits: BitsType,
}

impl CardMask {

    pub const fn new() -> Self {
        Self::from_bits(0)
    }

    pub const fn to_bits(self) -> BitsType {
        self.bits
    }

    pub const fn from_bits(bits: BitsType) -> Self {
        Self { bits: bits }
    }

    pub const fn from_card(card: Card) -> Self {
        // a single card is represented by a single bit set to 1
        Self::from_bits((1 as BitsType) << card.to_index())
    }

    pub const fn from_cards(cards: &[Card]) -> Self {
        let mut mask = Self::new();
        let mut i: usize = 0;
        while i < cards.len() {
            mask = mask.union(Self::from_card(cards[i]));
            i += 1;
        }
        mask
    }


    pub const fn from_suit(suit: Suit) -> Self {
        // a suit is represented by a single bit set to 1 for each card in the suit, which can be efficient represented
        // this can be done by subtracting 1 from a power of 2, and then shifting left by the suit index times the number of ranks (i.e. a big block of one-bits)
        Self::from_bits((((1 as BitsType) << Rank::NUM) - 1) << ((suit.to_index() as usize) * Rank::NUM))
    }
    
    pub const fn from_rank(rank: Rank) -> Self {
        // a rank is represented by a single bit set to 1 for each card in the rank, which are separated over all the suits, so not contiguous
        // this can be done in a different way (since it is not a block of one-bits)

        // first, construct a mask with a single bit set to 1 for each suit at the lowest rank (i.e. 2s)
        let mut bits: BitsType = 0;
        let mut i: usize = 0;
        while i < Suit::NUM {
            bits |= (1 as BitsType) << (i * Rank::NUM);
            i += 1;
        }

        // then, shift left by the rank index to offset it correctly
        bits <<= rank.to_index() as usize;
        
        // finally, return the mask
        Self::from_bits(bits)
    }

    pub const fn unsuited(&self) -> RankMask {
        // a suit-agnostic mask is a mask that ignores the suit of the cards, so we can use it to represent a rank set
        // to do this, just shift each suit's right
        let mut bits: BitsType = 0;

        let mut i: usize = 0;
        while i < Suit::NUM {
            let suit = Self::from_suit(Suit::from_index(i as u8));
            
            bits |= self.intersect(suit).to_bits() >> (i * Rank::NUM);
            i += 1;
        }

        RankMask::from_bits(bits as u32)
    }

    // an iterator for per-each-suit rank masks
    // pub const fn per_suit(&self) -> [(Suit, RankMask); Suit::NUM] {
    //     Suit::ALL.iter().map(|suit| (*suit, Self::from_suit(*suit).intersect(*self).unsuited())).collect::<Vec<(Suit, RankMask)>>()
    // }

    pub const fn union(&self, mask: Self) -> Self {
        Self::from_bits(self.bits | mask.bits)
    }

    pub const fn intersect(&self, mask: Self) -> Self {
        Self::from_bits(self.bits & mask.bits)
    }
    
    pub const fn difference(&self, mask: Self) -> Self {
        Self::from_bits(self.bits ^ mask.bits)
    }

    pub const fn empty(&self) -> bool {
        self.bits == 0
    }

    pub const fn count(&self) -> usize {
        self.bits.count_ones() as usize
    }

    pub fn iter(&self) -> impl Iterator<Item = Card> {
        // TODO: make this const
        (0..52).filter_map(|i| if (self.bits & (1 << i)) != 0 {
            Some(Card::from_index(i))
        } else {
            None
        })
    }

}

impl fmt::Display for CardMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CardMask::from(\"")?;
        for card in self.iter() {
            write!(f, "{}", card)?;
        }
        write!(f, "\")")?;
        Ok(())
    }
}

impl fmt::Debug for CardMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}



// TODO: RankMask and SuitMask as well?


// the base mask type, which can represent any set of cards
#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct RankMask {
    bits: u32,
}

impl RankMask {

    pub const fn new() -> Self {
        Self::from_bits(0)
    }

    pub const fn to_bits(self) -> u32 {
        self.bits
    }

    pub const fn from_bits(bits: u32) -> Self {
        Self { bits: bits }
    }

    pub const fn from_rank(rank: Rank) -> Self {
        Self::from_bits(1 << rank.to_index())
    }

    pub const fn from_ranks(ranks: &[Rank]) -> Self {
        let mut mask = Self::new();
        let mut i: usize = 0;
        while i < ranks.len() {
            mask = mask.union(Self::from_rank(ranks[i]));
            i += 1;
        }
        mask
    }

    pub const fn union(&self, mask: Self) -> Self {
        Self::from_bits(self.bits | mask.bits)
    }

    pub const fn intersect(&self, mask: Self) -> Self {
        Self::from_bits(self.bits & mask.bits)
    }

    pub const fn difference(&self, mask: Self) -> Self {
        Self::from_bits(self.bits ^ mask.bits)
    }

    pub const fn empty(&self) -> bool {
        self.bits == 0
    }

    pub const fn count(&self) -> usize {
        self.bits.count_ones() as usize
    }

    pub fn iter(&self) -> impl Iterator<Item = Rank> {
        (0..13).filter_map(|i| if (self.bits & (1 << i)) != 0 {
            Some(Rank::from_index(i))
        } else {
            None
        })
    }

}

impl fmt::Display for RankMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RankMask::from(\"")?;
        for rank in self.iter() {
            write!(f, "{:?}", rank)?;
        }
        write!(f, "\")")?;
        Ok(())
    }
}

impl fmt::Debug for RankMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
*/
