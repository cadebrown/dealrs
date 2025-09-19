//! Reference implementation for 5-card hand deduction
use crate::{deck::{CardMask, Rank, RankMask}, hand::{Best5, Hand}};

/// Reference implementation for 5-card hand deduction, which uses no data or lookup tables, and manually checks it analytically each time
pub struct RefBest5 {}

impl RefBest5 {

    pub fn new() -> Self {
        Self { }
    }

    /// Determine the best straight flush in the given set of cards, and return the cards used to make it as well as the top rank of the straight flush
    /// If no straight flush is found, return None
    pub fn best_straightflush(cards: CardMask) -> Option<(CardMask, Rank)> {
        cards.each_suit()
            .map(|(suit, _)| Self::best_straight(cards.intersect(CardMask::from_suit(suit))))
            .flatten()
            .max_by(|(_, a), (_, b)| a.to_index().cmp(&b.to_index()))
    }

    /// Determine the best four of a kind in the given set of cards, and return the cards used to make it as well as the rank of the quad
    /// If no four of a kind is found, return None
    pub fn best_fourofakind(cards: CardMask) -> Option<(CardMask, Rank)> {
        cards.each_rank()
            .filter(|(_, suit_mask)| suit_mask.count() >= 4)
            .map(|(rank, _)| rank)
            .max_by(|a, b| a.to_index().cmp(&b.to_index()))
            .map(|rank| (cards.intersect(CardMask::from_rank(rank)).top4(), rank))
    }

    /// Determine the best full house in the given set of cards, and return the cards used to make it as well as the ranks of the triple and pair
    /// If no full house is found, return None
    pub fn best_fullhouse(cards: CardMask) -> Option<(CardMask, (Rank, Rank))> {
        match Self::best_threeofakind(cards) {
            Some((used1, triple)) => {
                let rest = cards.intersect(used1.inverse());
                match Self::best_pair(rest) {
                    Some((used2, pair)) => {
                        return Some((used1.union(used2), (triple, pair)));
                    }
                    _ => {
                        None
                    }
                }
            }
            _ => {
                None
            }
        }
    }

    /// Determine the best straight in the given set of cards (determined by the order of the top 5 cards in the suit), and return the cards used as well the ranks of the straight
    /// If no straight is found, return None
    pub fn best_flush(cards: CardMask) -> Option<(CardMask, RankMask)> {
        cards.each_suit()
            .filter(|(_, ranks)| ranks.count() >= 5)
            .map(|(suit, ranks)| (suit, ranks.top5()))
            .max_by(|(_, a), (_, b)| a.to_bits().cmp(&b.to_bits()))
            .map(|(suit, ranks)| (cards.intersect(CardMask::from_suit(suit)).top5(), ranks))
    }

    /// Determine the best straight in the given set of cards, and return the cards used to make it as well as the top rank of the straight
    /// If no straight is found, return None
    pub fn best_straight(cards: CardMask) -> Option<(CardMask, Rank)> {
        // since straights are only defined by ranks, we can use the unsuited version of the cards to check for straights
        let ranks = cards.unsuited();

        // use bitwise magic to check for a straight
        let bits = ranks.to_bits();
        // bitwise convolution-like operation that checks for 5 consecutive bits set to 1
        let bits_conv5 = bits & (bits >> 1) & (bits >> 2) & (bits >> 3) & (bits >> 4);
        let is_straight = bits_conv5 != 0;
        if is_straight {
            // we have a striaght, but we need to determine which cards can be used to make it
            let top_rank_index = (64 - bits_conv5.leading_zeros() + 3) as u8;
            let mut used = CardMask::none();
            for i in 0..5 {
                let rank_top_sub_i = Rank::from_index(top_rank_index - i);
                // take an arbitrary card of the rank
                used = used.union(cards.intersect(CardMask::from_rank(rank_top_sub_i)).top1());
                assert!(used.count() == (i + 1) as usize, "failed for cards: {} -> used: {} -> i: {} -> rank_top_sub_i: {} -> top_rank_index: {}", cards, used, i, rank_top_sub_i, top_rank_index);
            }
            return Some((used, Rank::from_index(top_rank_index)));
        }

        // special case, we need to check for the 'wheel straight' (A2345), using a specific bitmask.
        // NOTE: this needs to be done last, since it is the lowest ranked straight
        if RankMask::from_multi(&[Rank::Ace, Rank::Two, Rank::Three, Rank::Four, Rank::Five]).intersect(cards.unsuited()).count() == 5 {
            let top_rank_index = Rank::Five.to_index();
            let mut used = CardMask::none();
            for i in 0..4 {
                let irank = Rank::from_index(top_rank_index - i);
                used = used.union(cards.intersect(CardMask::from_rank(irank)).top1());
                assert!(used.count() == (i + 1) as usize);
            }
            // special case: add the ace (which is considered low here)
            used = used.union(cards.intersect(CardMask::from_rank(Rank::Ace)).top1());
            assert!(used.count() == 5);
            return Some((used, Rank::Five));
        }

        // otherwise, no straight
        None
    }
    
    /// Determine the best three of a kind in the given set of cards, and return the cards used to make it as well as the rank of the triple
    /// If no three of a kind is found, return None
    pub fn best_threeofakind(cards: CardMask) -> Option<(CardMask, Rank)> {
        cards.each_rank()
            .filter(|(_, suit_mask)| suit_mask.count() >= 3)
            .map(|(rank, _)| rank)
            .max_by(|a, b| a.to_index().cmp(&b.to_index()))
            .map(|rank| (cards.intersect(CardMask::from_rank(rank)).top3(), rank))
    }

    /// Determine the best two pairs in the given set of cards, and return the cards used to make them, as well as the rank of the pairs
    /// If no two pairs are found, return None
    pub fn best_twopair(cards: CardMask) -> Option<(CardMask, (Rank, Rank))> {
        match Self::best_pair(cards) {
            Some((cards1, pair1)) => {
                let rest1 = cards.intersect(cards1.inverse());
                match Self::best_pair(rest1) {
                    Some((cards2, pair2)) => {
                        return Some((cards1.union(cards2), (pair1, pair2)));
                    }
                    _ => {
                        None
                    }
                }
            }
            _ => {
                None
            }
        }
    }

    /// Determine the best pair in the given set of cards, and return the cards used to make it as well as the rank of the pair
    /// If no pair is found, return None
    pub fn best_pair(cards: CardMask) -> Option<(CardMask, Rank)> {
        cards.each_rank()
            .filter(|(_, suit_mask)| suit_mask.count() >= 2)
            .map(|(rank, _)| rank)
            .max_by(|a, b| a.to_index().cmp(&b.to_index()))
            .map(|rank| (cards.intersect(CardMask::from_rank(rank)).top2(), rank))
    }

    pub fn best_high<const N: usize>(cards: CardMask) -> CardMask {
        cards.topn::<N>()
    }

}

impl Best5 for RefBest5 {

    fn best5(&self, cards: CardMask) -> (CardMask, Hand) {
        // do a chain of checks, in order of 'best' kind of hand (so that we can short-circuit as soon as we find the best hand)
        if let Some((used, rank)) = Self::best_straightflush(cards) {
            (used, Hand::StraightFlush { top: rank })
        } else if let Some((used, rank)) = Self::best_fourofakind(cards) {
            let rest = cards.intersect(used.inverse());
            let kicker = rest.unsuited().top1();
            (used, Hand::FourOfAKind { quad: rank, kickers: kicker })
        } else if let Some((used, rank)) = Self::best_fullhouse(cards) {
            (used, Hand::FullHouse { trip: rank.0, pair: rank.1 })
        } else if let Some((used, rank)) = Self::best_flush(cards) {
            (used, Hand::Flush { ranks: rank })
        } else if let Some((used, rank)) = Self::best_straight(cards) {
            (used, Hand::Straight { top: rank })
        } else if let Some((used, rank)) = Self::best_threeofakind(cards) {
            let rest = cards.intersect(used.inverse());
            (used, Hand::ThreeOfAKind { trip: rank, kickers: rest.unsuited() })
        } else if let Some((used, ranks    )) = Self::best_twopair(cards) {
            let rest = cards.intersect(used.inverse());
            (used, Hand::TwoPair { pairs: RankMask::from_multi(&[ranks.0, ranks.1]), kickers: rest.unsuited().top1() })
        } else if let Some((used, rank)) = Self::best_pair(cards) {
            let rest = cards.intersect(used.inverse());
            (used, Hand::OnePair { pair: rank, kickers: rest.unsuited().top3() })
        } else {
            let used = cards.top5();
            (used, Hand::HighCard { kickers: used.unsuited() })
        }
    }

}



// TODO: generic tests for multiple implementors of the trait?
#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_hand_eq {
        ($cards:expr, $expected:expr) => {
            let _cards = $cards.into();
            let _hr = RefBest5{}.best5(_cards);
            assert_eq!(_hr.1, $expected);
        };
    }

    macro_rules! assert_hand_ne {
        ($cards:expr, $expected:expr) => {
            let _cards = $cards.into();
            let _hr = RefBest5{}.best5(_cards);
            assert_ne!(_hr.1, $expected);
        };
    }

    macro_rules! assert_hand_lt {
        ($cards1:expr, $cards2:expr) => {
            let _cards1 = $cards1.into();
            let _cards2 = $cards2.into();
            let _hr1 = RefBest5{}.best5(_cards1);
            let _hr2 = RefBest5{}.best5(_cards2);
            if !(_hr1.1 < _hr2.1) {
                panic!("failed assertion: (lhs < rhs)\n\tlhs: {:?}\n\trhs: {:?}", _hr1.1, _hr2.1);
            }
        };
    }

    #[test]
    fn test_highcard() {
        let cards = "2s3h4s5h7s".into();
        let _hr = RefBest5{}.best5(cards);
        assert_eq!(_hr.1, Hand::HighCard { kickers: cards.unsuited() });

        assert_hand_eq!("2s3h4s5h7s", Hand::HighCard { kickers: "23457".into() });
        assert_hand_ne!("2s3h4s5h7s", Hand::HighCard { kickers: "2345".into() });
        assert_hand_ne!("2s3h4s5h7s", Hand::HighCard { kickers: "2".into() });

        assert_hand_eq!("2sAh4sJh7s", Hand::HighCard { kickers: "2A4J7".into() });
        assert_hand_ne!("2sAh4sJh7s", Hand::HighCard { kickers: "2A4J".into() });
        assert_hand_ne!("2sAh4sJh7s", Hand::HighCard { kickers: "2".into() });

        assert_hand_ne!("2s2h3c4d5s", Hand::HighCard { kickers: "22345".into() });
    }

    #[test]
    fn test_pair() {
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
        assert_hand_eq!("2s3h4s5h6s", Hand::Straight { top: Rank::Six });
        assert_hand_ne!("2s3h4s5h6s", Hand::Straight { top: Rank::Five });
    }
    
    #[test]
    fn test_flush() {
        assert_hand_eq!("2s3s4s5s7s", Hand::Flush { ranks: "23457".into() });
        assert_hand_ne!("2s3s4s5s7s", Hand::Flush { ranks: "2345".into() });
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
        assert_hand_ne!("2s3s4s5s6s", Hand::StraightFlush { top: Rank::Five });
    }


    #[test]
    fn test_cmp_hands() {
        assert_hand_lt!("2s3h4s5h7s", "2s3h4s5h8s");
        assert_hand_lt!("AsKhJsTh8s", "AsKhJsTh9s");
        assert_hand_lt!("2s3h4s5h7s", "2s3h4s5h6s");
        assert_hand_lt!("AsAhKsKhKc", "AsAhAcKhKc");
    }
}
