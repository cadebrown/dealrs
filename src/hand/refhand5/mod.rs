//! Reference implementation for 5-card hand deduction, with readable and understandable code
//! 
//! This is not necessarily the fastest implementation, but it is the most readable and understandable. It is also the most accurate, since it manually checks each possible hand.
//! 
//! This can be used with the LutRank implementation to generate a table once and reuse the faster indexing logic.

use crate::{deck::{CardMask, Rank, RankMask, Suit}, hand::{Hand, Hand5}};

/// Reference implementation for 5-card hand deduction, which uses no data or lookup tables, and manually checks it analytically each time
pub struct RefHand5 {}

impl RefHand5 {

    /// Create a new reference implementation for 5-card hand deduction, which is a no-op constructor
    pub fn new() -> Self {
        Self { }
    }

    /// Check ranks for a straight, returning the top rank if found
    fn check_straight(ranks: RankMask) -> Option<Rank> {
        let rbits = ranks.bits();

        // straight bits for each position in the rank mask, used for efficiency
        let sb4 = rbits << 0;
        let sb3 = rbits << 1;
        let sb2 = rbits << 2;
        let sb1 = rbits << 3;

        // special case for the wheel straight (A2345), which must count A as low
        // to do this, we include the top-shift-4, and the ace-shift-rest (which works because the ace is the highest rank, so no other ranks will be included) 
        let sb0 = (rbits << 4) | (rbits >> (13 - 4));

        // now, calculate the convolution of these rank bits, which corresponds to 1s at the top of consecutive runs
        let rbits_conv5 = sb4 & sb3 & sb2 & sb1 & sb0;
        if rbits_conv5 != 0 {
            // therefore, the highest rank is the highest bit set, which we can compute by leading zeros and some index adjustments
            let top_rank_index = (16 - 1 - rbits_conv5.leading_zeros()) as u8;
            Some(Rank::from_index(top_rank_index))
        } else {
            None
        }
    }

    /// Check cards for a flush, returning the top5 rank mask if found
    fn check_flush(cards: CardMask) -> Option<RankMask> {
        Suit::ALL.iter()
            .map(|&suit| cards.of_suit(suit).top5())
            .max()
    }

}

impl Hand5 for RefHand5 {

    fn hand5(&self, cards: CardMask) -> Hand {
        // get what unique ranks are present in the cards
        let ranks = cards.unsuited();

        // determine suitedness of the cards
        let mut have_suited5 = false;
        let mut suits_counts = [0; Suit::NUM];
        for suit in Suit::ALL {
            suits_counts[suit.index() as usize] = cards.of_suit(suit).count();
            if suits_counts[suit.index() as usize] >= 5 {
                have_suited5 = true;
            }
        }

        // keep track of the best straight we find
        let mut found_straight: Option<Rank> = None;

        // keep track of whether we actually checked for a straight
        let mut checked_straight: bool = false;

        // keep track of the best flush we find
        let mut found_flush: Option<RankMask> = None;

        if have_suited5 {

            // first, go ahead and perform our straight check
            found_straight = Self::check_straight(ranks);
            checked_straight = true;

            // then, if we have a straight, we need to check for a straight flush
            if let Some(_) = found_straight {

                // keep track of the best straight flush we find
                // TODO: there is a faster way to do this, by checking all at once with a compound bitmask
                let mut best_straight_flush: Option<Rank> = None;
                for suit in Suit::ALL {
                    if suits_counts[suit.index() as usize] >= 5 {
                        if let Some(top_rank) = Self::check_straight(cards.of_suit(suit)) {
                            if best_straight_flush.is_none() || top_rank > best_straight_flush.unwrap() {
                                best_straight_flush = Some(top_rank);
                            }
                        }
                    }
                }


                // if we found a straight flush, return it as it is the best hand possible
                if let Some(top_rank) = best_straight_flush {
                    return Hand::StraightFlush { top: top_rank };
                }
            } 

            // otherwise, we might as well go ahead and check for a flush
            found_flush = Self::check_flush(cards);
        }

        let mut best_trip: Option<Rank> = None;
        let mut best_pair1: Option<Rank> = None;
        let mut best_pair2: Option<Rank> = None;
        // now, we can check for quads and full houses. we do this by scanning through the ranks in reverse order, and keeping track of what we see
        for rank in Rank::ALL.iter().rev() {
            // check for just this rank
            // let cards_of_rank = cards.of_rank(*rank);
            let cards_of_rank_count = cards.of_rank_count(*rank);
            if cards_of_rank_count >= 4 {
                // we can go ahead and return a four of a kind, since nothing can beat it
                return Hand::FourOfAKind { quad: *rank, kickers: (cards.unsuited() & RankMask::from(*rank).inverse()).top1() };
            } else if cards_of_rank_count >= 3 {
                match best_pair1 {
                    Some(pair) => {
                        // we have a full house with the best trip and best pair, so return it
                        return Hand::FullHouse { trip: *rank, pair: pair };
                    }
                    None => {
                        // we have a trip, so keep track of it
                        best_trip = Some(*rank);
                    }
                }
            } else if cards_of_rank_count >= 2 {
                // similar logic, but for pairs
                match best_trip {
                    Some(trip) => {
                        return Hand::FullHouse { trip: trip, pair: *rank };
                    }
                    _ => {}
                }

                // and, lets check two pairs
                match best_pair1 {
                    Some(pair) => {
                        match best_pair2 {
                            Some(pair2) => { }
                            _ => {
                                best_pair2 = Some(*rank);
                            }
                        }
                    }
                    _ => {
                        best_pair1 = Some(*rank);
                    }
                }
            }
        }

        if let Some(flush) = found_flush {
            return Hand::Flush { ranks: flush };
        }

        // now, check for a straight if we haven't already
        if !checked_straight {
            found_straight = Self::check_straight(ranks);
        }

        if let Some(straight) = found_straight {
            return Hand::Straight { top: straight };
        }

        if let Some(trip) = best_trip {
            assert!(best_pair1.is_none(), "should not have a pair if we have a trip, that should have been caught earlier as a full house");
            return Hand::ThreeOfAKind { trip: trip, kickers: (cards.unsuited() & RankMask::from(trip).inverse()).top2() };
        }
        if let Some(pair1) = best_pair1 {
            match best_pair2 {
                Some(pair2) => {
                    return Hand::TwoPair { pairs: RankMask::from_many(&[pair1, pair2]), kickers: (cards.unsuited() & RankMask::from_many(&[pair1, pair2]).inverse()).top1() };
                }
                _ => {
                    return Hand::OnePair { pair: pair1, kickers: (cards.unsuited() & RankMask::from(pair1).inverse()).top3() };
                }
            }
        }

        // if we didn't find anything, return a high card
        Hand::HighCard { kickers: ranks.top5() }
    }

}

// TODO: generic tests for multiple implementors of the trait?
#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_hand_eq {
        ($cards:expr, $expected:expr) => {
            let _cards = $cards.into();
            let _hr = RefHand5{}.hand5(_cards);
            assert_eq!(_hr, $expected);
        };
    }

    macro_rules! assert_hand_ne {
        ($cards:expr, $expected:expr) => {
            let _cards = $cards.into();
            let _hr = RefHand5{}.hand5(_cards);
            assert_ne!(_hr, $expected);
        };
    }

    macro_rules! assert_hand_lt {
        ($cards1:expr, $cards2:expr) => {
            let _cards1 = $cards1.into();
            let _cards2 = $cards2.into();
            let _hr1 = RefHand5{}.hand5(_cards1);
            let _hr2 = RefHand5{}.hand5(_cards2);
            if !(_hr1.1 < _hr2.1) {
                panic!("failed assertion: (lhs < rhs)\n\tlhs: {:?}\n\trhs: {:?}", _hr1.1, _hr2.1);
            }
        };
    }

    #[test]
    fn test_highcard() {
        let cards = "2s3h4s5h7s".into();
        let _hr = RefHand5{}.hand5(cards);
        assert_eq!(_hr, Hand::HighCard { kickers: cards.unsuited() });

        assert_hand_eq!("2s3h4s5h7s", Hand::HighCard { kickers: "23457".into() });
        assert_hand_ne!("2s3h4s5h7s", Hand::HighCard { kickers: "2345".into() });
        assert_hand_ne!("2s3h4s5h7s", Hand::HighCard { kickers: "2".into() });

        assert_hand_eq!("2sAh4sJh7s", Hand::HighCard { kickers: "2A4J7".into() });
        assert_hand_ne!("2sAh4sJh7s", Hand::HighCard { kickers: "2A4J".into() });
        assert_hand_ne!("2sAh4sJh7s", Hand::HighCard { kickers: "2".into() });

        assert_hand_ne!("2s2h3c4d5s", Hand::HighCard { kickers: "22345".into() });
    }

    #[test]
    fn test_onepair() {
        assert_hand_eq!("2s2h3c4d5s", Hand::OnePair { pair: Rank::Two, kickers: "345".into() });
        assert_hand_ne!("2s2h3c4d5s", Hand::OnePair { pair: Rank::Two, kickers: "34".into() });

        assert_hand_eq!("AsAh3c4d5s", Hand::OnePair { pair: Rank::Ace, kickers: "345".into() });
        assert_hand_ne!("AsAh3c4d5s", Hand::OnePair { pair: Rank::Ace, kickers: "34".into() });

        assert_hand_ne!("AsAh3c3d5s", Hand::OnePair { pair: Rank::Ace, kickers: "345".into() });
        assert_hand_ne!("AsAh3cAd5s", Hand::OnePair { pair: Rank::Ace, kickers: "34".into() });
    }

    #[test]
    fn test_twopair() {
        assert_hand_eq!("2s2h3c3d5s", Hand::TwoPair { pairs: "23".into(), kickers: "5".into() });
        assert_hand_ne!("2s2h3c3d5s", Hand::TwoPair { pairs: "23".into(), kickers: "56".into() });
    }

    #[test]
    fn test_threeofakind() {
        assert_hand_eq!("2s2h2c3d5s", Hand::ThreeOfAKind { trip: Rank::Two, kickers: "35".into() });
        assert_hand_ne!("2s2h2c3d5s", Hand::ThreeOfAKind { trip: Rank::Two, kickers: "356".into() });
    }

    #[test]
    fn test_straight() {
        assert_hand_eq!("AsKhQsJhTs", Hand::Straight { top: Rank::Ace });
        assert_hand_eq!("2s3h4s5h6s", Hand::Straight { top: Rank::Six });
        assert_hand_eq!("2s3h4s5hAs", Hand::Straight { top: Rank::Five });
        assert_hand_eq!("2s3h4s5hAcKhQdJhTc", Hand::Straight { top: Rank::Ace });
        assert_hand_eq!("2s3h4s5hAsKhQsJh", Hand::Straight { top: Rank::Five });
        assert_hand_eq!("2s3h4d5hAsKdQdJh6s", Hand::Straight { top: Rank::Six });
    }
    
    #[test]
    fn test_flush() {
        assert_hand_eq!("2s3s4s5s7s", Hand::Flush { ranks: "23457".into() });
        assert_hand_ne!("2s3s4s5s7s", Hand::Flush { ranks: "2345".into() });
        assert_hand_eq!("2s3s4s5s7sTsQs", Hand::Flush { ranks: "QT754".into() });

        assert_hand_eq!("2c5c8c7c7s9cTcTs", Hand::Flush { ranks: "T9785".into() });
    }

    #[test]
    fn test_fullhouse() {
        assert_hand_eq!("2s2h2c3d3s", Hand::FullHouse { trip: Rank::Two, pair: Rank::Three });
        assert_hand_ne!("2s2h2c3d3s", Hand::FullHouse { trip: Rank::Two, pair: Rank::Four });
    }
    
    #[test]
    fn test_fourofakind() {
        assert_hand_eq!("2s2h2c2d5s", Hand::FourOfAKind { quad: Rank::Two, kickers: "5".into() });
        assert_hand_ne!("2s2h2c2d5s", Hand::FourOfAKind { quad: Rank::Two, kickers: "56".into() });
    }
    
    #[test]

    fn test_straightflush() {
        assert_hand_eq!("2s3s4s5s6s", Hand::StraightFlush { top: Rank::Six });
        assert_hand_eq!("6s7s8s9sTs", Hand::StraightFlush { top: Rank::Ten });
        assert_hand_eq!("As2s3s4s5s", Hand::StraightFlush { top: Rank::Five });
        assert_hand_eq!("KsQsJsTs9s", Hand::StraightFlush { top: Rank::King });

        // test weird wrapping, wheel, and royal flushes
        assert_hand_eq!("AsKsQsJsTs2s3s4s5s6s", Hand::StraightFlush { top: Rank::Ace });
        assert_hand_eq!("AsQsJsTs2s3s4s5s", Hand::StraightFlush { top: Rank::Five });
        assert_hand_eq!("AsQsJsTs2s3s4s5s6s", Hand::StraightFlush { top: Rank::Six });
    }

}
