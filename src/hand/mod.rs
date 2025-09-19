//! Hand detection, evaluation, and ranking for poker hands

use crate::deck::{Rank, CardMask, RankMask};

use core::fmt;

pub mod refbest5;
pub mod lutbest5;

/// A trait describing engines capable of evaluating the best 5-card subset of a hand, returning the cards used and the hand specification it generates
pub trait Best5 {

    /// Determine the overall best hand, using the best 5 cards in the given set of cards possible, returning the cards used as well as the overall best hand
    fn best5(&self, cards: CardMask) -> (CardMask, Hand);
}

/// A trait describing engines capable of ranking a hand into a comparable
pub trait Rank5 {
    
    /// Rank the given hand, giving an ordering number that is comparable to other hands. Higher numbers are better.
    /// TODO: specify how it handles fewer-than-5 cards (idea1: fill in the kickers with the lowest ranks, such that the hand is absolutely guaranteed to improve to at least the rank returned by that logic) (idea2: expand the canonical hand ranking to include incomplete hands, which burns index space in tables but allows for comparing incomplete hands and generalizes better)
    fn rank5(&self, cards: CardMask) -> u32;

}

/// Represents all possible poker hands, predelineated in order of strength (i.e. directly comparable), which can be used to convert raw cards into a hand
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum Hand {

    /// A high card hand, which matches no other hand defined here. The kickers are used to break ties, of which there can be up to 5
    /// For example, "2,3,4,5,7" is a high card, but "2,3,4,5,6" is not (it fulfills the requirements for a straight)
    HighCard {
        kickers: RankMask,
    },

    /// A (single) pair, which requires 2 cards of the same rank. The kickers are used to break ties, of which there can be up to 3
    OnePair {
        pair: Rank,
        kickers: RankMask,
    },

    /// A two pair, which requires 2 pairs of the 2 separate ranks. The kicker (if it exists) is used to break ties, of which there can be up to 1
    TwoPair {
        pairs: RankMask,
        kickers: RankMask,
    },

    /// A three of a kind (trip), which requires 3 cards of the same rank. The kickers are used to break ties, of which there can be up to 2
    ThreeOfAKind {
        trip: Rank,
        kickers: RankMask,
    },

    /// A straight, which requires a consecutive sequence of 5 cards in any suit (A is high and low). Therefore, the top rank is the only factor in breaking ties between different straights
    Straight {
        top: Rank,
    },

    /// A flush, which requires at least 5 cards of the same suit. Only the top 5 cards are used to break ties, thus lower ranks are ignored upon ties in all 5 top ranks
    Flush {
        ranks: RankMask,
    },

    /// A full house, which requires 3 cards of the same rank and 2 cards of another rank (i.e. 3-of-a-kind and a pair). The trip is more important than the pair in determining ties between different full houses
    FullHouse {
        trip: Rank,
        pair: Rank,
    },

    /// A four of a kind, which requires 4 cards of the same rank.  The kickers will decide ties between different hands with the same primary rank
    FourOfAKind {
        quad: Rank,
        kickers: RankMask,
    },

    /// A straight flush, which requires a consecutive sequence of 5 cards in the same suit (A is high and low). Therefore, the top rank is the only factor in breaking ties between different straight flushes
    StraightFlush {
        top: Rank,
    },

}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HighCard { kickers } => {
                write!(f, "Kickers: '{:}'", kickers)
            },
            Self::OnePair { pair, kickers } => {
                write!(f, "Pair '{:}' + Kickers '{:}'", pair, kickers)
            },
            Self::TwoPair { pairs, kickers } => {
                write!(f, "Two Pair '{:}' + Kickers '{:}'", pairs, kickers)
            },
            Self::ThreeOfAKind { trip, kickers } => {
                write!(f, "Trip '{:}' + Kickers '{:}'", trip, kickers)
            },
            Self::Straight { top } => {
                match top {
                    Rank::Five => {
                        write!(f, "Straight 'A2345'")
                    },
                    _ => {
                        write!(f, "Straight '")?;
                        for i in 0..5 {
                            let irank = Rank::from_index(top.to_index() - i);
                            write!(f, "{}", irank)?;
                            if i > 0 {
                                write!(f, "")?;
                            }
                        }
                        write!(f, "'")
                    }
                }
            },
            Self::FourOfAKind { quad, kickers } => {
                write!(f, "Quad '{:}' + Kickers '{:}'", quad, kickers)
            },
            Self::FullHouse { trip, pair } => {
                write!(f, "Full House '{:}{:}'", trip, pair)
            },
            Self::Flush { ranks } => {
                write!(f, "Flush '{:}'", ranks)
            },
            Self::StraightFlush { top } => {
                match top {
                    Rank::Five => {
                        write!(f, "Straight Flush 'A2345'")
                    },
                    _ => {
                        write!(f, "Straight Flush '")?;
                        for i in 0..5 {
                            let irank = Rank::from_index(top.to_index() - i);
                            write!(f, "{}", irank)?;
                            if i > 0 {
                                write!(f, "")?;
                            }
                        }
                        write!(f, "'")
                    }
                }
            },
        }
    }
}
