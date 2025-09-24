//! Hand detection, evaluation, and ranking for standard 5-cards-or-fewer poker hands. These are sufficient for standard game variants like Texas Holdem, Omaha Holdem, and 5/7-card Stud/Draw.
//! 
//! There are 2 main things you want to do with hands, which correspond to the 2 main traits in this module:
//! 
//! * [Hand5]: determine the best possible hand from a given set of cards using up to 5 cards, returning a structured type that fully describes the hand
//!   * This constructs a [Hand] object that can be matched against, or used to display to the user
//!   * Additionally, results can be sorted and compared to each other, through Rust's built-in comparison operators
//! * [Rank5]: rank a hand into a comparable number, such that hands can be sorted and compared
//!   * This is useful for generating a unique number for a hand, which can be used to compare, sort, or reference hands
//!   * This is often faster to use in practice, especially for large sets of hands in simulations
//!  
//! ## Engine Implementations
//! 
//! We include a few different engines, which may work better for different use cases. You can implement your own, and in the future we may provide more specialized implementations.
//! 
//! ### [RefHand5](refhand5) - Reference Implementation
//! 
//! This is the reference implementation for the [Hand5] trait, which is the most accurate and comprehensive implementation. It is also the slowest, so it is not recommended for large scale simulations.
//! 
//! ### [LutRank](lutrank) - Look-Up Table Implementation
//! 
//! This is a fast implementation of the [Rank5] trait, which uses a look-up table to determine the rank of a hand. It is less accurate than the reference implementation, but is much faster.
//!  
//! This can be used to avoid startup costs at the expense of binary size, but it is typically pretty small.
//! 
//! TODO: include by default with a feature flag?
//! 

// use crate::deck::{Rank, CardMask, RankMask};

use crate::deck::{CardMask, Rank, RankMask};

use serde::{Deserialize, Serialize};

use core::fmt;

pub mod refhand5;
pub mod lutrank;

/// A trait for engines capable of evaluating the best 5-card-or-fewer subset of a hand, returning the hand specification it generates
/// 
/// TODO: specify how engines with restrictions (like LUT with maximum hand sizes) should handle cases they don't include? And a wrapper that falls back to reference implementation?
pub trait Hand5 {

    /// Determine the overall best 5-card-or-fewer poker hand, using the cards that were given. Every engine must support this for up to 5 cards.
    /// 
    /// TODO: specify limits of the engine? for now, they may panic but this isn't very nice
    fn hand5(&self, cards: CardMask) -> Hand;

}
/// A trait for engines capable of producing a ranking order for the best 5-card-or-fewer subset of a hand, returning an integer that is comparable and should always be equivalent to calling `Best5` and comparing the resulting `Hand`s (but, often will be faster)
/// 
/// This is super useful in cases where you don't need to display or care about the specifics of the hand, but just the order. For instance, equity calculations or large simulations that produce a lot of hands will like to use the smaller storage size and faster operations (and, graphing/charting is more trivial to produce axes)
/// 
/// TODO: also include a RankToHand5 trait that can convert a ranking number into a hand? This should be easy, but unclear if it should be a hard requirement, or if multiple implementations will always agree...
pub trait Rank5 {
    
    /// Rank the given hand, giving an ordering number that is comparable to other hands. Higher numbers are better.
    /// 
    /// TODO: specify how it handles fewer-than-5 cards (idea1: fill in the kickers with the lowest ranks, such that the hand is absolutely guaranteed to improve to at least the rank returned by that logic) (idea2: expand the canonical hand ranking to include incomplete hands, which burns index space in tables but allows for comparing incomplete hands and generalizes better)
    fn rank5(&self, cards: CardMask) -> u16;

}

/// Represents a standard poker hand using 5-cards-or-fewer. These are comparable with the same order of "which is better" used to score hands in most variants.
/// 
/// WARNING: Don't construct these manually! You can easily create an 'invalid' hand (like HighCard of "AKQJT", which should be a straight!) that will cause subtle errors. Instead, use a hand-determiner that implements `Best5`. For example, use `RefBest5{}` for a reference implementation that's very fast and tested for correctness!
/// 
/// Each category of the hand corresponds to a category in the [standard list of poker hands](https://en.wikipedia.org/wiki/List_of_poker_hands). Within each category, the different parameters are listed, in order of their importance for the hand. For instance, a full house is first evaluated based on the card that has 3x (called the 'trip'), and then the additional pair (so, 'KKK33' > 'QQQAA')
/// 
/// There are also 'incomplete' hands, which represents a hand that definitely does fit the category, but not enough cards were given to fill out the auxiliary parameters. For example, consider getting pocket aces in the hole in Texas Holdem ('AA'). If you evaluate this hand, you should get a OnePair of 'A' with kickers of '???'. This can be interpreted as a lower bound to the mininum hand that those cards will improve to when any other cards are added to it. 
/// 
/// Something to keep in mind is that, even though (Pair 'A' + Kickers '???') is guaranteed to improve to at least (Pair 'A' + Kickers '234') on the flop, the 'incomplete' hand is still considered distinct, and will compare as a "worse" hand than the complete hand. This can be useful in many scenarios of simulations, as well as variants that may not even use 5 full cards. In cases where you track hands over a round of a game, always recompute the rank of the hand at the end of the game to avoid subtle errors.
/// 
/// TODO: make methods that construct the 'minumum complete' hand after some number of cards are added to it?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Hand {

    /// A high card hand, which matches no other hand defined here. The kickers are used to break ties, of which there can be up to 5
    /// 
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
                let mut kickers = kickers.iter_reverse();
                let top_kicker: Option<Rank> = kickers.next();
                let rest = kickers.collect::<Vec<_>>();
                write!(f, "High Card '{:}' with Kickers '{:}{:}'", top_kicker.map(|r| r.to_string()).unwrap_or("?".to_string()), RankMask::from_many(&rest), "?".repeat(4 - rest.len()))
            },
            Self::OnePair { pair, kickers } => {
                write!(f, "One Pair of '{:}' with Kickers '{:}{:}'", pair, kickers, "?".repeat(3 - kickers.count()))
            },
            Self::TwoPair { pairs, kickers } => {
                let pairs = pairs.iter_reverse().collect::<Vec<_>>();
                write!(f, "Two Pair of '{:}' over '{:}' with Kickers '{:}{:}'", pairs[0], pairs[1], kickers, "?".repeat(1 - kickers.count()))
            },
            Self::ThreeOfAKind { trip, kickers } => {
                write!(f, "Three of a Kind of '{:}' with Kickers '{:}{:}'", trip, kickers, "?".repeat(2 - kickers.count()))
            },
            Self::Straight { top } => {
                write!(f, "Straight to '{:}'", top)
            },
            Self::FourOfAKind { quad, kickers } => {
                write!(f, "Four of a Kind of '{:}' with Kickers '{:}{:}'", quad, kickers, "?".repeat(1 - kickers.count()))
            },
            Self::FullHouse { trip, pair } => {
                write!(f, "Full House of '{:}' over '{:}'", trip, pair)
            },
            Self::Flush { ranks } => {
                write!(f, "Flush with '{:}'", ranks)
            },
            Self::StraightFlush { top } => {
                write!(f, "Straight Flush to '{:}'", top)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_highcard() {
        assert_eq!("High Card '?' with Kickers '????'", format!("{}", Hand::HighCard { kickers: RankMask::NONE }));
        assert_eq!("High Card '7' with Kickers '5432'", format!("{}", Hand::HighCard { kickers: "2,3,4,5,7".into() }));
        assert_eq!("High Card '7' with Kickers '52??'", format!("{}", Hand::HighCard { kickers: "2,5,7".into() }));
        assert_eq!("High Card 'A' with Kickers 'K???'", format!("{}", Hand::HighCard { kickers: "A,K".into() }));
        assert_eq!("High Card 'A' with Kickers '52??'", format!("{}", Hand::HighCard { kickers: "A,5,2".into() }));
    }

    #[test]
    fn display_onepair() {
        assert_eq!("One Pair of '2' with Kickers '???'", format!("{}", Hand::OnePair { pair: "2".into(), kickers: RankMask::NONE }));
        assert_eq!("One Pair of '2' with Kickers '754'", format!("{}", Hand::OnePair { pair: "2".into(), kickers: "4,5,7".into() }));
        assert_eq!("One Pair of '2' with Kickers '7??'", format!("{}", Hand::OnePair { pair: "2".into(), kickers: "7".into() }));
        assert_eq!("One Pair of 'A' with Kickers 'K52'", format!("{}", Hand::OnePair { pair: "A".into(), kickers: "K,5,2".into() }));
        assert_eq!("One Pair of 'J' with Kickers 'T??'", format!("{}", Hand::OnePair { pair: "J".into(), kickers: "T".into() }));
    }

    #[test]
    fn display_twopair() {
        assert_eq!("Two Pair of '3' over '2' with Kickers '?'", format!("{}", Hand::TwoPair { pairs: "2,3".into(), kickers: RankMask::NONE }));
        assert_eq!("Two Pair of 'K' over 'T' with Kickers '3'", format!("{}", Hand::TwoPair { pairs: "K,T".into(), kickers: "3".into() }));
    }


    #[test]
    fn display_threeofakind() {
        assert_eq!("Three of a Kind of '2' with Kickers '??'", format!("{}", Hand::ThreeOfAKind { trip: "2".into(), kickers: RankMask::NONE }));
        assert_eq!("Three of a Kind of '2' with Kickers '75'", format!("{}", Hand::ThreeOfAKind { trip: "2".into(), kickers: "5,7".into() }));
        assert_eq!("Three of a Kind of '2' with Kickers '7?'", format!("{}", Hand::ThreeOfAKind { trip: "2".into(), kickers: "7".into() }));
        assert_eq!("Three of a Kind of 'A' with Kickers 'K2'", format!("{}", Hand::ThreeOfAKind { trip: "A".into(), kickers: "K,2".into() }));
        assert_eq!("Three of a Kind of 'J' with Kickers 'T?'", format!("{}", Hand::ThreeOfAKind { trip: "J".into(), kickers: "T".into() }));
    }

    #[test]
    fn display_straight() {
        // special case for the wheel straight
        assert_eq!("Straight to '5'", format!("{}", Hand::Straight { top: Rank::Five }));
        assert_eq!("Straight to 'A'", format!("{}", Hand::Straight { top: Rank::Ace }));
        assert_eq!("Straight to '9'", format!("{}", Hand::Straight { top: Rank::Nine }));
        assert_eq!("Straight to 'Q'", format!("{}", Hand::Straight { top: Rank::Queen }));
    }

    #[test]
    fn display_flush() {
        assert_eq!("Flush with 'AT543'", format!("{}", Hand::Flush { ranks: "3,4,5,T,A".into() }));
        assert_eq!("Flush with 'KT853'", format!("{}", Hand::Flush { ranks: "3,5,8,T,K".into() }));
    }


    #[test]
    fn display_fullhouse() {
        assert_eq!("Full House of 'A' over 'K'", format!("{}", Hand::FullHouse { trip: Rank::Ace, pair: Rank::King }));
        assert_eq!("Full House of 'K' over 'A'", format!("{}", Hand::FullHouse { trip: Rank::King, pair: Rank::Ace }));
    } 

    #[test]
    fn display_fourofakind() {
        assert_eq!("Four of a Kind of '2' with Kickers '?'", format!("{}", Hand::FourOfAKind { quad: Rank::Two, kickers: RankMask::NONE }));
        assert_eq!("Four of a Kind of '2' with Kickers '7'", format!("{}", Hand::FourOfAKind { quad: Rank::Two, kickers: "7".into() }));
        assert_eq!("Four of a Kind of 'J' with Kickers 'T'", format!("{}", Hand::FourOfAKind { quad: Rank::Jack, kickers: "T".into() }));
    }

    #[test]
    fn display_straightflush() {
        assert_eq!("Straight Flush to '5'", format!("{}", Hand::StraightFlush { top: Rank::Five }));
        assert_eq!("Straight Flush to 'A'", format!("{}", Hand::StraightFlush { top: Rank::Ace }));
        assert_eq!("Straight Flush to '9'", format!("{}", Hand::StraightFlush { top: Rank::Nine }));
        assert_eq!("Straight Flush to 'Q'", format!("{}", Hand::StraightFlush { top: Rank::Queen }));
    }


    macro_rules! assert_lt {
        ($lhs:expr, $rhs:expr) => {
            assert!($lhs < $rhs);
        };
    }


    #[test]
    fn compare_hands() {
        assert_eq!(
            Hand::HighCard { kickers: "2,3,4,5,7".into() },
            Hand::HighCard { kickers: "2,3,4,5,7".into() }
        );
        assert_ne!(
            Hand::HighCard { kickers: "2,3,4,5,8".into() },
            Hand::HighCard { kickers: "2,3,4,5,7".into() }
        );
        assert_lt!(
            Hand::HighCard { kickers: "2,3,4,5".into() },
            Hand::HighCard { kickers: "2,3,4,5,8".into() }
        );

        assert_lt!(
            Hand::HighCard { kickers: "3,4,5,8".into() },
            Hand::HighCard { kickers: "2,3,4,5,8".into() }
        );

        assert_lt!(
            Hand::HighCard { kickers: "A,K,Q,J,9".into() },
            Hand::OnePair { pair: Rank::Two, kickers: RankMask::NONE}
        );

        assert_lt!(
            Hand::TwoPair { pairs: "A,K".into(), kickers: RankMask::NONE },
            Hand::ThreeOfAKind { trip: Rank::Two, kickers: RankMask::NONE }
        );

        assert_lt!(
            Hand::ThreeOfAKind { trip: Rank::Two, kickers: RankMask::NONE },
            Hand::Straight { top: Rank::Three }
        );

        assert_lt!(
            Hand::Straight { top: Rank::Three },
            Hand::Flush { ranks: "A,K,Q,J,9".into() }
        );

        assert_lt!(
            Hand::Flush { ranks: "A,K,Q,J,9".into() },
            Hand::FullHouse { trip: Rank::Two, pair: Rank::Three }
        );
        assert_lt!(
            Hand::FullHouse { trip: Rank::Two, pair: Rank::Three },
            Hand::FourOfAKind { quad: Rank::Two, kickers: RankMask::NONE }
        );
        assert_lt!(
            Hand::FourOfAKind { quad: Rank::Two, kickers: RankMask::NONE },
            Hand::StraightFlush { top: Rank::Three }
        );
        assert_lt!(
            Hand::StraightFlush { top: Rank::Three },
            Hand::StraightFlush { top: Rank::Ace }
        );

    }


    #[test]
    fn compare_hands_partial() {
        // handle partially constructed hands

        assert_eq!(
            Hand::HighCard { kickers: "".into() },
            Hand::HighCard { kickers: "".into() }
        );

        assert_lt!(
            Hand::HighCard { kickers: "".into() },
            Hand::HighCard { kickers: "2".into() }
        );

        assert_lt!(
            Hand::HighCard { kickers: "2,3,4,5".into() },
            Hand::HighCard { kickers: "2,3,4,5,8".into() }
        );
    }

}

