//! A Look-Up-Table (LUT) implementation for the canonical rankings of all 5-card hands, useful for fast hand evaluation, sorting, comparison, and validation of other LUT structures
//! 
//! By 'canonical', we mean that there are only a finite number of hands possible, and in fact only a few distinct hands when considering comparison to other hands. This is capped at 5 cards, since for example only the 5 cards that best contribute to a hand are considered when determining ties.
//! 
//! This canonical LUT is concerned with all possible hands using 5 or fewer cards, meaning that it also allows for 'unknown' cards/ranks (represented by '?' in the descriptions). These can be considered 'incomplete' hands, in that in most games they will not be used at the end of the game, but instead will become a complete hand after more cards are revealed.
//! 
//! However, these incomplete hands are still useful for many purposes, such as equity calculations, displaying current game state for a player's POV, or even games with fewer than 4 total cards. Incomplete hands have a few useful properties:
//! 
//! * in games that reveal more cards, incomplete hands are a minimum bound to the strength of the hand you will have at the end of the game
//! * they consider missing kickers as the lowest ranks, so even a 'complete' hand with a 2 kicker will be considered higher than an 'incomplete' hand with a missing kicker
//! 
//! This can be slightly misleading in the scenario where an incomplete hand is NOT considered equivalent to the minimum hand it is guaranteed to improve to. This means, for accurate tie resolution, you need to recompute the rank of the hand at the end of the game if it was previously incomplete.
//! 

use std::{collections::BTreeMap, io::Write};

use kdam::tqdm;

use itertools::Itertools;

use serde::{Deserialize, Serialize};

// use crate::{combrs::{multiset_decode, multiset_encode}, deck::{Card, CardMask, Rank, Suit}, hand::{refbest5::RefBest5, Best5, Hand, Rank5}};
use crate::{ combrs::bagspace::{BagSpace, SetSpace}, deck::{Card, CardMask, Rank, RankMask, Suit}, hand::{refhand5::RefHand5, Hand5, Hand, Rank5}};


/// The keys are a special structure using combinatorial encoding to represent the ranks of the cards in the hand
#[derive(Debug, Serialize, Deserialize)]
pub struct LutRank {

    /// Lookup table to use when all cards in the hand are of the same suit (i.e. flushes and straight flushes)
    pub allsuited: Vec<u16>,

    /// Lookup table to use when cards of different suits are present (i.e. all other hands)
    pub nonsuited: Vec<u16>,

    /// Lookup table to use when determining the hand from a rank, used so that it is more compact (since, there are often skips in the other LUTs)
    pub orders2hands: Vec<Hand>,

    /// The bag space to use for encoding and decoding the ranks of the cards in the hand, which are not neccessarily of a single suit
    pub bagspace_nonsuited: BagSpace,

    /// The set space of all suited cards (i.e. no repetion of ranks)
    pub setspace_allsuited: SetSpace,

}

impl LutRank {

    /// Create a new lookup table from the built-in lookup tables
    pub fn new() -> Self {
        // TODO: use a cfg/feature flag to determine whether to use the built-in lookup tables or to compute them from scratch
        // maybe also a caching function?
        Self::from_brute_force()
    }

    /// Create a new lookup table from brute force computation, which is slow but accurate and can be used to verify or recreate the lookup tables
    pub fn from_brute_force() -> Self {

        // create a reference engine to use for verification
        let engine_ref = RefHand5::new();

        // keep track of all the hands generated (in comparison order), and the cards used to generate them
        // NOTE: it shouldn't matter if multiple map to the same hand
        let mut bts_hands = BTreeMap::<Hand, Vec<u64>>::new();

        // the maximum number of ranks to consider
        // TODO: make this generic, and allow different LUTs
        let max_ranks = 7;

        // now, create a vector of all the ranks, and a None for the last position (so, we effectively encode all sequences of 5/4/3/2/1/0 ranks)
        // we will need to give care to this later
        let mut ranks_space = Rank::ALL.iter().rev()
            .map(|&rank| Some(rank))
            .chain([None])
            .collect::<Vec<Option<Rank>>>();

        // reverse the whole thing, so it is in the correct order
        ranks_space.reverse();

        assert!(ranks_space.len() == 14, "expected 14 ranks (13 normal + 1 for empty)");

        // construct the sampling space
        let bagspace_nonsuited = BagSpace::new(ranks_space.len(), max_ranks);
        let setspace_allsuited = SetSpace::new(ranks_space.len());

        // now, let's iterate over all possible sequences
        for ranks in tqdm!(ranks_space.iter().combinations_with_replacement(max_ranks)) {

            // check for 5-of-a-kind, and exit early if we have it
            if ranks.iter()
                .chunk_by(|&rank| rank)
                .into_iter()
                .any(|(value, chunk)| value.is_some() && chunk.count() >= 5) 
            {
                continue;
            }

            // now, we try to build a plausible hand from the sequence
            // convert to raw integer sequence
            let mut seq = vec![0usize; max_ranks];
            let mut i = 0;
            let mut cards = CardMask::NONE;
            for rank in ranks.iter() {
                if let Some(rank) = rank {
                    // get the suits that are NOT present for this rank
                    let not_of_rank = !cards.of_rank(*rank);
                    assert!(not_of_rank.count() > 0, "expected at least card left");

                    // pick the suit with the least number of cards
                    let suit = not_of_rank.iter()
                        .min_by_key(|&suit| cards.of_suit(suit).count())
                        .unwrap();
                    assert!(cards.of_suit(suit).count() < 4, "expected suit to have less than 4 cards");

                    // and construct a card
                    let card = Card::new(*rank, suit);
                    assert!(!cards.contains(card.into()), "expected card not to be in the mask");

                    // add it to the mask
                    cards = cards | card.into();

                    seq[i] = (1 + rank.index()) as usize;
                    i += 1;
                } else {
                    seq[i] = 0;
                    i += 1;
                }
            }

            assert!(i == seq.len(), "expected sequence to be complete");
            assert!(seq.len() == bagspace_nonsuited.num_seq, "expected sequence to be complete");
            assert!(seq.is_sorted(), "expected sequence to be sorted");

            // and, encode it as a key
            let key_nonsuited = bagspace_nonsuited.enc::<usize, usize>(&seq);
            let hand_nonsuited = engine_ref.hand5(cards);

            // insert or append to the list of keys for this hand
            bts_hands.entry(hand_nonsuited).or_insert(vec![]).push(key_nonsuited as u64);

            if cards.unsuited().count() >= 5 {
                // if we can, turn it into all suited cards for a suited hand as well with the same ranks

                // now, turn the ranks into all suited cards
                let cards = cards.unsuited().iter()
                    .fold(CardMask::NONE, |acc, rank| acc | Card::new(rank, Suit::Hearts).into());
                assert!(cards.count() >= 5, "expected at least 5 cards");

                // replace seq with all 0s
                for i in 0..max_ranks {
                    seq[i] = 0;
                }

                // now, store the unsuited ranks at the end
                for (idx, rank) in cards.unsuited().iter_reverse().enumerate() {
                    seq[max_ranks - idx - 1] = (1 + rank.index()) as usize;
                }

                // and, encode it as a key
                // let key_allsuited = setspace_allsuited.enc::<usize, usize>(&seq);
                let key_allsuited = cards.unsuited().bits() as usize;
                let hand_allsuited = engine_ref.hand5(cards);
                bts_hands.entry(hand_allsuited).or_insert(vec![]).push(key_allsuited as u64);
            }
        }

        // the maximum key index
        let max_key_nonsuited = bts_hands.iter().map(|(_, keys)| keys.iter().max().unwrap()).max().unwrap();
        let max_order = bts_hands.len() + 1;
        let max_key_allsuited = (1 << 13) - 1;

        let mut allsuited_ranks = vec![0u16; (max_key_allsuited + 1) as usize];
        let mut nonsuited_ranks = vec![0u16; (max_key_nonsuited + 1) as usize];
        let mut orders2hands = vec![Hand::HighCard { kickers: RankMask::NONE }; (max_order + 1) as usize];

        for (order, (&hand, keys)) in bts_hands.iter().enumerate() {

            // skip over 0, and start the order at 1 so that 0 can be used for special cases (i.e. no hand, not found, etc)
            let order = order + 1;

            // now, we just insert the hand into the appropriate category
            match hand {
                Hand::StraightFlush { .. } | Hand::Flush { .. } => {
                    // all-suited hands (only flushes and straight flushes) go into the corresponding LUT
                    for key in keys {
                        // store the order and corresponding hand used for it
                        allsuited_ranks[*key as usize] = order as u16;
                        orders2hands[order as usize] = hand;
                    }
                }
                _ => {
                    // non-suited hands (i.e. everything else) go into the corresponding LUT
                    for key in keys {
                        // store the order and corresponding hand used for it
                        nonsuited_ranks[*key as usize] = order as u16;
                        orders2hands[order as usize] = hand;
                    }
                }
            }
        }

        Self { allsuited: allsuited_ranks, nonsuited: nonsuited_ranks, orders2hands, bagspace_nonsuited, setspace_allsuited }

        // // we create 2 distinct 'categories' of hands: all-suited and non-suited and track them
        // //   * 'all-suited' -> all 5 cards are of the same suit, thus there are only flushes and straight flushes, and no ranks are repeated
        // //   * 'non-suited' -> there are 2 or more suits, thus all other hands are possible
        // let mut allsuited = IndexMap::new();
        // let mut nonsuited = IndexMap::new();

        // // to populate these separate categories, we just iterate over all hands (in sorted/ranked order), and use the index as their canonical key
        // // we still separate them into the 2 categories, so that querying is faster and easier to determine based on suitedness
        // for (order, (hand, &key)) in bts_hands.iter().enumerate() {

        //     // to determine the 'canonical' ranking index of the hand, we just use the raw sorted index, plus 1 to make it 1-based in case we want to use 0 for special cases (i.e. no hand, for Rust's Option optimizations)
        //     let rank_index = order + 1;

        //     // the key used in the lookup table is the mask of cards used to generate it, but with suits removed/ignored. this works because we separate on suitedness
        //     // further, when querying, this allows us to strip suit information earlier and reduce the key space
        //     // let key = key;

        //     // now, we just insert the hand into the appropriate category
        //     match hand {
        //         Hand::StraightFlush { .. } | Hand::Flush { .. } => {
        //             // put suited hands in the all suited lookup table
        //             allsuited.insert(key, (rank_index as u16, hand));
        //         }
        //         _ => {
        //             // put non-suited hands in the all suited lookup table
        //             nonsuited.insert(key, (rank_index as u16, hand));
        //         }
        //     }
        // }


        // let key_max = allsuited.iter()
        //     .map(|(&h,_)| h)
        //     .chain(nonsuited.iter().map(|(&h,_)| h)).max().unwrap();

        // let mut allsuited = vec![0u16; (key_max + 1) as usize];
        // let mut nonsuited = vec![0u16; (key_max + 1) as usize];

        // let mut allsuited_hands = vec![Hand::HighCard { kickers: RankMask::NONE }; (key_max + 1) as usize];
        // let mut nonsuited_hands = vec![Hand::HighCard { kickers: RankMask::NONE }; (key_max + 1) as usize];

        // for (i, (&flat_idx, (rank_idx, hand))) in allsuited.iter().enumerate() {
        //     allsuited[flat_idx as usize] = (i + 1) as u16;
        //     allsuited_hands[flat_idx as usize] = **hand;
        // }
        // for (i, (&flat_idx, (rank_idx, hand))) in nonsuited.iter().enumerate() {
        //     nonsuited[flat_idx as usize] = (i + 1) as u16;
        //     nonsuited_hands[flat_idx as usize] = **hand;
        // }
        // Self { allsuited: allsuited_flat, nonsuited: nonsuited_flat, bagspace, allsuited_hands, nonsuited_hands }
    }


    // pub fn ranks2key(&self, ranks: &[usize]) -> u64 {
    //     let mut ranks_int = vec![0; 5];
    //     for (idx, &rank) in ranks.iter().enumerate() {
    //         ranks_int[idx] = 1 + (rank as usize);
    //     }
    //     self.bagspace.enc::<usize, u64>(&ranks_int)
    // }

    // pub fn ranks2key(&self, ranks: RankMask) -> u64 {
    //     // the raw sequence of ranks used for multiset encoding
    //     let mut seq = [0usize; 8];

    //     let mut i = 0;

    //     for rank in ranks.iter_reverse() {
    //         seq[self.bagspace.num_seq - i - 1] = (1 + rank.index()) as usize;
    //         i += 1;
    //     }

    //     println!("seq: {:?}", seq);

    //     self.bagspace.enc::<usize, u64>(&seq)
    // }

    pub fn cards2key(&self, cards: CardMask) -> u64 {

        // the raw sequence of ranks used for multiset encoding
        let mut seq = [0usize; 7];
        let mut i = 0;
        for &rank in Rank::ALL.iter().rev() {
            for _ in 0..(cards.of_rank_count(rank)) {
                seq[self.bagspace_nonsuited.num_seq - i - 1] = (1 + rank.index()) as usize;
                i += 1;
            }
        }
        let idx = self.bagspace_nonsuited.enc::<usize, u64>(&seq);
        idx
    }
    pub fn ranks2key(&self, ranks: RankMask) -> u64 {

        // the raw sequence of ranks used for multiset encoding
        let mut seq = [0usize; 7];
        let mut i = 0;
        for rank in ranks.iter_reverse() {
            seq[self.bagspace_nonsuited.num_seq - i - 1] = (1 + rank.index()) as usize;
            i += 1;
        }
        let idx = self.bagspace_nonsuited.enc::<usize, u64>(&seq);
        idx
    }

    // pub fn find_hand(&self, cards: CardMask) -> Option<Hand> {
    //     let key = self.cards2key(cards);
    //     // Some(*self.nonsuited_hands.get(key as usize).unwrap())

    // }


    // pub fn find_order(&self, cards: CardMask) -> Option<u16> {
    //     let key = self.cards2key(cards);
    //     self.nonsuited.get(key as usize).map(|&x| x)
    // }

    /// Rank exactly 5 cards
    pub fn find(&self, cards: CardMask) -> Option<u16> {
        // Some(*self.nonsuited.get(key as usize).unwrap())

        let mut max_res: Option<u16> = None;

        Suit::ALL.iter().for_each(|&suit| {
            let ranks = cards.of_suit(suit);
            if ranks.count() >= 5 {
                // let key = self.ranks2key(ranks);
                let key = ranks.bits() as usize;
                max_res = max_res.max(self.allsuited.get(key).map(|&x| x));
            }
        });

        // and finally, do non-suited
        let key = self.cards2key(cards);
        max_res = max_res.max(self.nonsuited.get(key as usize).map(|&x| x));

        max_res



        // let key = self.cards2key(cards);
// 
        // if false {
        //     Some(*self.allsuited.get(key as usize).unwrap())
        // } else {
        //     Some(*self.nonsuited.get(key as usize).unwrap())
        // }


        // assert_eq!(cards.count(), 5, "expected exactly 5 cards");
        // let bits = cards.bits();

        // // TODO: less hacky?
        // let bits_suit = 0b1111111111111;
        // let is_suited = (bits & bits_suit == bits) || ((bits & (bits_suit << 13) == bits)) || ((bits & (bits_suit << 26) == bits)) || ((bits & (bits_suit << 39) == bits));


    }

    pub fn write_markdown(&self, writer: &mut impl Write) -> Result<(), Box<dyn std::error::Error>> {

        // print the header
        writeln!(writer, "# Ordered Ranking of All 5-card Hands")?;
        writeln!(writer, "")?;
        writeln!(writer, "This file contains the ordered rankings of all 5-card hands, sorted by rank. The ranks are 1-based, with 0 being used for special cases (i.e. no hand).")?;
        writeln!(writer, "")?;
        writeln!(writer, "## Non-Suited 5-card Hands")?;
        writeln!(writer, "")?;
        writeln!(writer, "| Order | Description                                      |")?;
        writeln!(writer, "|-------|--------------------------------------------------|")?;
        

        for (order, hand) in self.orders2hands.iter().enumerate() {
            if order == 0 {
                continue;
            }
            match hand {
                Hand::StraightFlush { .. } | Hand::Flush { .. } => {
                }
                _ => {
                    writeln!(writer, "| {:>5} | {:<48} |", order, hand.to_string())?;
                }
            }
        }


        // let mut order_nonsuited = self.nonsuited.iter()
        //     .filter(|&order| *order > 0)
        //     .map(|&order| (order, self.orders2hands[order as usize]))
        //     .sorted_by_key(|(order, _)| *order)
        //     .collect::<Vec<(u16, Hand)>>();

        // for (order, hand) in order_nonsuited {
        //     writeln!(writer, "| {:>5} | {:<48} |", order, hand.to_string())?;
        // }

        writeln!(writer, "|-------|--------------------------------------------------|")?;
        writeln!(writer, "")?;
        writeln!(writer, "## All-Suited 5-card Hands")?;
        writeln!(writer, "")?;
        writeln!(writer, "| Order | Description                                      |")?;
        writeln!(writer, "|-------|--------------------------------------------------|")?;

        for (order, hand) in self.orders2hands.iter().enumerate() {
            if order == 0 {
                continue;
            }
            match hand {
                Hand::StraightFlush { .. } | Hand::Flush { .. } => {
                    writeln!(writer, "| {:>5} | {:<48} |", order, hand.to_string())?;
                }
                _ => {
                }
            }
        }

        // let mut order_allsuited = self.allsuited.iter()
        //     .filter(|&order| *order > 0)
        //     .map(|&order| (order, self.orders2hands[order as usize]))
        //     .sorted_by_key(|(order, _)| *order)
        //     .collect::<Vec<(u16, Hand)>>();

        // for (order, hand) in order_allsuited {
        //     writeln!(writer, "| {:>5} | {:<48} |", order, hand.to_string())?;
        // }

        writeln!(writer, "|-------|--------------------------------------------------|")?;

        Ok(())
    }

}

impl Rank5 for LutRank {
    fn rank5(&self, cards: CardMask) -> u16 {
        // assert_eq!(cards.count(), 5, "expected exactly 5 cards");
        self.find(cards).unwrap()
    }
}

impl Hand5 for LutRank {
    fn hand5(&self, cards: CardMask) -> Hand {
        match self.find(cards) {
            Some(order) => self.orders2hands[order as usize],
            _ => self.orders2hands[0]
        }
    }
}


// #[cfg(test)]
// mod tests {
//     use crate::combrs::binom;

//     use super::*;

//     #[test]
//     fn test_multiset_encdec() {

//         // first, iterate over all possible sequences and make sure they can be encoded and decoded back to the original sequence
//         let n = 13;
//         let k = 5;
//         for seq in (0..n).combinations_with_replacement(k) {
//             let mut seq = seq.iter().map(|x| *x as usize).collect::<Vec<usize>>();
//             seq.sort();
//             let idx: usize = multiset_encode(&seq);
//             let mut decoded = vec![0; k];
//             multiset_decode(idx, n, k, &mut decoded);
//             assert_eq!(seq, decoded);
//         }

//         // now, iterate over all possible indices and make sure they can be decoded back to the original sequence
//         for idx in 0..binom(n+k-1, k) {
//             let mut seq = vec![0; k];
//             multiset_decode(idx, n, k, &mut seq);
//             let idx_redo: usize = multiset_encode(&seq);
//             assert_eq!(idx, idx_redo);
//         }

//     }
// }


/*
// a helper structure for quickly searching the lookup tables
#[derive(Debug)]
pub struct LutCard5 {
    lut5_allsuited: indexmap::IndexMap<u64, u16>,
    lut5_nonsuited: indexmap::IndexMap<u64, u16>,
}

impl LutCard5 {

    fn key5_mask(cards: CardMask) -> u64 {
        assert_eq!(cards.count(), 5, "expected exactly 5 cards");

        let mut key = 0;
        for (rank, rank_count) in cards.each_rank().collect::<Vec<(Rank, SuitMask)>>().iter().rev().map(|(rank, rank_count)| (rank, rank_count.count())) {
            for _ in 0..rank_count {
                key = 13 * key + (rank.to_index() as u64);
            }
        }
        key
    }

    fn key5_text(text: &str) -> u64 {
        assert!(text.len() == 5, "expected exactly 5 cards");
        let mut key = 0;
        for c in text.chars() {
            let cindex = match c {
                '2' => Rank::Two.to_index(),
                '3' => Rank::Three.to_index(),
                '4' => Rank::Four.to_index(),
                '5' => Rank::Five.to_index(),
                '6' => Rank::Six.to_index(),
                '7' => Rank::Seven.to_index(),
                '8' => Rank::Eight.to_index(),
                '9' => Rank::Nine.to_index(),
                'T' => Rank::Ten.to_index(),
                'J' => Rank::Jack.to_index(),
                'Q' => Rank::Queen.to_index(),
                'K' => Rank::King.to_index(),
                'A' => Rank::Ace.to_index(),
                _ => panic!("invalid character: {}", c),
            };
            key = 13 * key + (cindex as u64);
        }
        key
    }

    fn key2str(key: u64) -> String {
        // keep dividing by 13 and getting the remainder
        let mut key_left = key;
        let mut str = String::new();
        for _ in 0..5 {
            str = str + &Rank::from_index((key_left % 13) as u8).to_string();
            key_left /= 13;
        }
        str.chars().rev().collect()
    }

    pub fn new_from_builtin() -> Self {
        Self {
            lut5_allsuited: indexmap::IndexMap::from_iter(
                serde_json::from_str::<Value>(&LUTCARD5_ALLSUITED).unwrap().as_object().unwrap().iter().map(|(k, v)| (Self::key5_text(k), v.as_u64().unwrap() as u16))),
            lut5_nonsuited: indexmap::IndexMap::from_iter(
                serde_json::from_str::<Value>(&LUTCARD5_NONSUITED).unwrap().as_object().unwrap().iter().map(|(k, v)| (Self::key5_text(k), v.as_u64().unwrap() as u16))),
        }
    }

    /// Rank exactly 5 cards
    pub fn find5(&self, cards: CardMask) -> Option<u16> {
        assert_eq!(cards.count(), 5, "expected exactly 5 cards");

        let is_suited = cards.each_suit().any(|(_, ranks)| ranks.count() >= 5); 

        let key = Self::key5_mask(cards);

        if is_suited {
            self.lut5_allsuited.get(&key).cloned()
        } else {
            self.lut5_nonsuited.get(&key).cloned()
        }
    }

    /// Forcibly create a new lookup table from scratch
    pub fn new_compute() -> Self {
        
        // create sets of hands, inserting into them will automatically sort them
        let mut bts_hands = BTreeMap::<Hand, CardMask>::new();

        // iterate over all possible 5-card hands
        for cards in tqdm!(Card::ALL.iter().combinations(5)) {
            // convert to a mask of cards
            let mut mask = CardMask::none();
            for card in cards {
                mask = mask.union(CardMask::from_single(*card));
            }

            // determine the hand
            let hand = Hand::from_cards(mask);

            // insert the hand into the lookup table 
            bts_hands.insert(hand, mask);

        }

        // now, convert to indices using their order in the sets
        let mut lut5_allsuited = IndexMap::new();
        let mut lut5_nonsuited = IndexMap::new();
        for (i, (hand, mask)) in bts_hands.iter().enumerate() {
            // convert to 1-based index, so that 0 can be used for special cases
            let rank_index = i + 1;
            let key = Self::key5_mask(*mask);
            match hand {
                Hand::StraightFlush { .. } | Hand::Flush { .. } => {
                    // put suited hands in the all suited lookup table
                    lut5_allsuited.insert(key, rank_index as u16);
                }
                _ => {
                    // put non-suited hands in the all suited lookup table
                    lut5_nonsuited.insert(key, rank_index as u16);
                }
            }
        }
        Self { lut5_allsuited, lut5_nonsuited }
    }

    pub fn as_json(&self) -> (String, String) {
        // convert them to JSON strings
        let mut str_allsuited = String::new();
        let mut str_nonsuited = String::new();
        str_allsuited += "{\n";

        for (i, (key, value)) in self.lut5_allsuited.iter().enumerate() {
            if i > 0 {
                str_allsuited += ",\n";
            }
            str_allsuited += &format!("    \"{}\": {}", Self::key2str(*key), value);
        }
        str_allsuited += "\n}";
        str_nonsuited += "{\n";
        for (i, (key, value)) in self.lut5_nonsuited.iter().enumerate() {
            if i > 0 {
                str_nonsuited += ",\n";
            }
            str_nonsuited += &format!("    \"{}\": {}", Self::key2str(*key), value);
        }
        str_nonsuited += "\n}";
        (str_allsuited, str_nonsuited)
    }
}
 */
