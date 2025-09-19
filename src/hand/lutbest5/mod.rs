//! A lookup table implementation to efficiently determine the best 5-card hand from a set of cards, useful for fast hand evaluation

use std::{collections::BTreeMap, str::FromStr, io::Write};

use indexmap::IndexMap;
use kdam::tqdm;

use itertools::Itertools;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{combrs::{multiset_decode, multiset_encode}, deck::{Card, CardMask, Rank, Suit}, hand::{refbest5::RefBest5, Best5, Hand, Rank5}};

// include the lookup table for all 5-card hands of exactly 1 suit (i.e. flushes and straight flushes)
// TODO: precompile into actual binary, or Rust code?
#[cfg(feature = "include_lutbest5")]
const LUTBEST5_TEXT: &str = include_str!("lutbest5.json");

#[cfg(not(feature = "include_lutbest5"))]
const LUTBEST5_TEXT: &str = "";

/// The 'key' type used for looking up hands in the lookup table, which corresponds to an unsuited mask of cards as an integer, or a string of 5 characters representing the ranks of the cards
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyLutCard5 {
    /// The raw data, compressed into a single integer
    data: u64,
}

impl KeyLutCard5 {
    pub fn from_index(index: usize) -> Self {
        Self { data: index as u64 }
    }
    pub fn from_cards(cards: CardMask) -> Self {
        // assert_eq!(cards.count(), 5, "expected exactly 5 cards");
        // let mut ranks = cards.iter().map(|c| c.rank().to_index() as usize).collect::<Vec<usize>>();
        // ranks.sort();
        // assert_eq!(ranks.len(), 5, "expected exactly 5 ranks");
        // let index = multiset_encode::<usize, usize>(&ranks);
        // Self { data: index as u64 }

        assert_eq!(cards.count(), 5, "expected exactly 5 cards");
        // TODO: clean this up??
        let mut seq = vec![0; 5];
        let mut rankmask = 0b1;
        let mut pos = 0;
        rankmask = rankmask | (rankmask << 13) | (rankmask << 26) | (rankmask << 39);
        for rankidx in 0..13 {
            for _ in 0..(cards.to_bits() & rankmask).count_ones() {
                seq[pos] = rankidx;
                if pos == 4 {
                    break;
                }
                pos += 1;
            }
            rankmask = rankmask << 1;
        }
        let index: usize = multiset_encode(&seq);
        Self::from_index(index)
    }

    pub fn to_rank_str(&self) -> String {
        let mut strs =Vec::new();

        let mut items = vec![0; 5];
        multiset_decode(self.data as usize, Rank::NUM, 5, &mut items);
        for &decnum in items.iter() {
            strs.push(Rank::from_index(decnum as u8).to_string());
        }
        strs.reverse();
        strs.join("")
    }

    pub fn to_rank_vec(&self) -> Vec<Rank> {
        let mut items = vec![0; 5];
        multiset_decode(self.data as usize, Rank::NUM, 5, &mut items);
        items.iter().map(|&decnum| Rank::from_index(decnum as u8)).collect()
    }

}


impl Serialize for KeyLutCard5 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_rank_str())
    }
}

impl<'de> Deserialize<'de> for KeyLutCard5 {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let mut ranks = s.chars().map(|r| Rank::from_str(&r.to_string()).unwrap().to_index() as usize).collect::<Vec<usize>>();
        ranks.sort();

        Ok(KeyLutCard5 { data: multiset_encode(&ranks) })
    }
}


/// Describes the lookup table for 5-card hands, which is a map of keys to ranks
/// The keys are a special structure using combinatorial encoding to represent the ranks of the cards in the hand
#[derive(Debug, Serialize, Deserialize)]
pub struct LutBest5 {

    /// Lookup table to use when all cards in the hand are of the same suit (i.e. flushes and straight flushes)
    pub allsuited: Vec<u16>,

    /// Lookup table to use when cards of different suits are present (i.e. all other hands)
    pub nonsuited: Vec<u16>,

}

impl LutBest5 {

    /// Create a new lookup table from the built-in lookup tables
    pub fn new() -> Self {
        // TODO: use a cfg/feature flag to determine whether to use the built-in lookup tables or to compute them from scratch
        // maybe also a caching function?
        Self::from_builtin()
    }

    /// Create a new lookup table from the built-in lookup tables
    pub fn from_builtin() -> Self {
        // just load it from the plain string
        // TODO: add sanity checks, and add a preload option?
        serde_json::from_str::<Self>(&LUTBEST5_TEXT).unwrap()
    }

    /// Create a new lookup table from brute force computation, which is slow but accurate and can be used to verify or recreate the lookup tables
    pub fn from_brute_force() -> Self {

        // create a reference engine to use for verification
        let engine_ref = RefBest5::new();

        // keep track of all the hands generated (in comparison order), and the cards used to generate them
        // NOTE: it shouldn't matter if multiple map to the same hand
        let mut bts_hands = BTreeMap::<Hand, KeyLutCard5>::new();

        // now, iterate over all possible 5-card hands, only somewhat naively
        // since there are 52 distinct cards, this is (52 choose 5) = 2598960 hands
        // but, we can just consider them as combinations with replacements over the ranks ((13 + 5 - 1) choose 5) = 1287, and rule out 5-of-a-kind, and do suited/nonsuited separately
        for ranks in tqdm!(Rank::ALL.iter().combinations_with_replacement(5)) {

            // skip 5-of-a-kind, since we can only have 4-replacements each
            if ranks.iter().all_equal() {
                continue;
            }

            // create masks for the 'allsuited' and 'nonsuited' cases
            let mut mask_allsuited = CardMask::none();
            let mut mask_nonsuited = CardMask::none();
            for (idx, &&rank) in ranks.iter().enumerate() {
                // for the non-suited case, just keep adding new cards of hearts
                mask_allsuited = mask_allsuited.union(CardMask::from_single(Card::new(rank, Suit::Hearts)));

                // now, for the non-suited case we need to add new different suits each time. this can get tricky since we could have duplicate ranks, so we can't just choose static ones each time
                let mut tries = 0;
                while mask_nonsuited.count() < idx+1 {
                    // didn't work, so try the next suit
                    let suit_for_nonsuited = match (idx + tries) % 4 {
                        0 => Suit::Hearts,
                        1 => Suit::Diamonds,
                        2 => Suit::Clubs,
                        3 => Suit::Spades,
                        _ => panic!("invalid suit index: {}", idx),
                    };
                    mask_nonsuited = mask_nonsuited.union(CardMask::from_single(Card::new(rank, suit_for_nonsuited)));
                    tries += 1;
                    if tries > 4 {
                        panic!("failed to find a suit for the non-suited case");
                    }
                }
            }

            // if the all-suited hand was possible (i.e. no duplicates), insert it
            if mask_allsuited.count() == 5 {
                bts_hands.insert(engine_ref.best5(mask_allsuited).1, KeyLutCard5::from_cards(mask_allsuited));
            }

            // always insert the non-suited hand
            bts_hands.insert(engine_ref.best5(mask_nonsuited).1, KeyLutCard5::from_cards(mask_nonsuited));
        }

        // we create 2 distinct 'categories' of hands: all-suited and non-suited and track them
        //   * 'all-suited' -> all 5 cards are of the same suit, thus there are only flushes and straight flushes, and no ranks are repeated
        //   * 'non-suited' -> there are 2 or more suits, thus all other hands are possible
        let mut allsuited = IndexMap::new();
        let mut nonsuited = IndexMap::new();

        // to populate these separate categories, we just iterate over all hands (in sorted/ranked order), and use the index as their canonical key
        // we still separate them into the 2 categories, so that querying is faster and easier to determine based on suitedness
        for (rawidx, (hand, &key)) in bts_hands.iter().enumerate() {

            // to determine the 'canonical' ranking index of the hand, we just use the raw sorted index, plus 1 to make it 1-based in case we want to use 0 for special cases (i.e. no hand, for Rust's Option optimizations)
            let rank_index = rawidx + 1;

            // the key used in the lookup table is the mask of cards used to generate it, but with suits removed/ignored. this works because we separate on suitedness
            // further, when querying, this allows us to strip suit information earlier and reduce the key space
            // let key = key;

            // now, we just insert the hand into the appropriate category
            match hand {
                Hand::StraightFlush { .. } | Hand::Flush { .. } => {
                    // put suited hands in the all suited lookup table
                    allsuited.insert(key, rank_index as u16);
                }
                _ => {
                    // put non-suited hands in the all suited lookup table
                    nonsuited.insert(key, rank_index as u16);
                }
            }
        }

        // flatten the lookup tables
        let max_idx_allsuited = allsuited.iter().map(|(&h,_)| h.data).max().unwrap();
        let max_idx_nonsuited = nonsuited.iter().map(|(&h,_)| h.data).max().unwrap();
        let mut allsuited_flat = vec![0u16; (max_idx_allsuited + 1) as usize];
        let mut nonsuited_flat = vec![0u16; (max_idx_nonsuited + 1) as usize];
        for (i, (&flat_idx, _)) in allsuited.iter().enumerate() {
            allsuited_flat[flat_idx.data as usize] = (i + 1) as u16;
        }
        for (i, (&flat_idx, _)) in nonsuited.iter().enumerate() {
            nonsuited_flat[flat_idx.data as usize] = (i + 1) as u16;
        }
        Self { allsuited: allsuited_flat, nonsuited: nonsuited_flat }
    }

    /// Rank exactly 5 cards
    pub fn find(&self, cards: CardMask) -> Option<u16> {
        assert_eq!(cards.count(), 5, "expected exactly 5 cards");
        let bits = cards.to_bits();

        // TODO: less hacky?
        let bits_suit = 0b1111111111111;
        let is_suited = (bits & bits_suit == bits) || ((bits & (bits_suit << 13) == bits)) || ((bits & (bits_suit << 26) == bits)) || ((bits & (bits_suit << 39) == bits));

        // convert to a key that is used by the lookup table directly
        let key = KeyLutCard5::from_cards(cards);
        if is_suited {
            Some(*self.allsuited.get(key.data as usize).unwrap())
        } else {
            Some(*self.nonsuited.get(key.data as usize).unwrap())
        }
    }

    pub fn write_markdown(&self, writer: &mut impl Write) -> Result<(), Box<dyn std::error::Error>> {

        let engine_ref = RefBest5::new();

        // print the header
        writeln!(writer, "# Rankings of All 5-card Hands")?;
        writeln!(writer, "")?;
        writeln!(writer, "This file contains the rankings of all 5-card hands, sorted by rank. The ranks are 1-based, with 0 being used for special cases (i.e. no hand).")?;
        writeln!(writer, "")?;
        writeln!(writer, "## Non-Suited 5-card Hands")?;
        writeln!(writer, "")?;
        writeln!(writer, "| Rank  | Cards | Description                  |")?;
        writeln!(writer, "|-------|-------|------------------------------|")?;

        let mut arr_nonsuited = self.nonsuited.iter()
            .map(|&x| x)
            .enumerate()
            .filter(|(_, x)| *x > 0)
            .collect::<Vec<(usize, u16)>>();
        arr_nonsuited.sort_by_key(|(_, rankidx)| *rankidx);
        for (cardsidx, rankidx) in arr_nonsuited {
            let cards = KeyLutCard5::from_index(cardsidx);

            // tricky: generate possible cards that are not-suited
            let mut cards_possible = CardMask::none();
            for (which_rank, &rank) in cards.to_rank_vec().iter().enumerate() {
                let mut tries = 0;
                while cards_possible.count() < which_rank + 1 {
                    // didn't work, so try the next suit
                    let suit_for_nonsuited = match (which_rank + tries) % 4 {
                        0 => Suit::Hearts,
                        1 => Suit::Diamonds,
                        2 => Suit::Clubs,
                        3 => Suit::Spades,
                        _ => panic!("invalid suit index: {}", tries),
                    };
                    cards_possible = cards_possible.union(CardMask::from_single(Card::new(rank, suit_for_nonsuited)));
                    tries += 1;
                    if tries > 4 {
                        panic!("failed to find a suit for the non-suited case");
                    }
                }
            }
            assert_eq!(cards, KeyLutCard5::from_cards(cards_possible));
            assert_eq!(cards_possible.count(), 5);
            // now, construct the hand from a reference implementation
            let hand_ref = engine_ref.best5(cards_possible).1;

            writeln!(writer, "| {:>5} | {:>5} | {:<28} |", rankidx, cards.to_rank_str(), hand_ref.to_string())?;
        }
        writeln!(writer, "|-------|-------|------------------------------|")?;
        writeln!(writer, "")?;
        writeln!(writer, "## All-Suited 5-card Hands")?;
        writeln!(writer, "")?;
        writeln!(writer, "| Rank  | Cards | Description                  |")?;
        writeln!(writer, "|-------|-------|------------------------------|")?;

        let mut arr_allsuited = self.allsuited.iter()
        .map(|&x| x)
        .enumerate()
        .filter(|(_, x)| *x > 0)
        .collect::<Vec<(usize, u16)>>();
        arr_allsuited.sort_by_key(|(_, rankidx)| *rankidx);
        for (cardsidx, rankidx) in arr_allsuited {
            let cards = KeyLutCard5::from_index(cardsidx);

            let mut cards_possible = CardMask::none();
            for &rank in cards.to_rank_vec().iter() {
                cards_possible = cards_possible.union(CardMask::from_single(Card::new(rank, Suit::Hearts)));
            }
            assert_eq!(cards, KeyLutCard5::from_cards(cards_possible));
            assert_eq!(cards_possible.count(), 5);
            let hand_ref = engine_ref.best5(cards_possible).1;
            writeln!(writer, "| {:>5} | {:>5} | {:<28} |", rankidx, cards.to_rank_str(), hand_ref.to_string())?;
        }

        Ok(())
    }
}

impl Rank5 for LutBest5 {
    fn rank5(&self, cards: CardMask) -> u32 {
        if cards.count() == 5 {
            self.find(cards).unwrap() as u32
        } else {
            // otherwise, iterate over all possible 5-card subsets and find the maximum one
            assert!(cards.count() <= 15);
            let mut best_rank = 0;

            let mut cards_bits = [CardMask::none(); 15];
            for (idx, card) in cards.iter().enumerate() {
                cards_bits[idx] = CardMask::from_single(card);
            }
            for subset in cards_bits[..cards.count()].iter().combinations(5) {
                let mut subset_mask = CardMask::none();
                for &bits in subset {
                    subset_mask = subset_mask.union(bits);
                }
                assert_eq!(subset_mask.count(), 5);
                let rank = self.find(subset_mask).unwrap() as u32;
                if rank > best_rank {
                    best_rank = rank;
                }
            }
            best_rank
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::combrs::binom;

    use super::*;

    #[test]
    fn test_multiset_encdec() {

        // first, iterate over all possible sequences and make sure they can be encoded and decoded back to the original sequence
        let n = 13;
        let k = 5;
        for seq in (0..n).combinations_with_replacement(k) {
            let mut seq = seq.iter().map(|x| *x as usize).collect::<Vec<usize>>();
            seq.sort();
            let idx: usize = multiset_encode(&seq);
            let mut decoded = vec![0; k];
            multiset_decode(idx, n, k, &mut decoded);
            assert_eq!(seq, decoded);
        }

        // now, iterate over all possible indices and make sure they can be decoded back to the original sequence
        for idx in 0..binom(n+k-1, k) {
            let mut seq = vec![0; k];
            multiset_decode(idx, n, k, &mut seq);
            let idx_redo: usize = multiset_encode(&seq);
            assert_eq!(idx, idx_redo);
        }

    }
}


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
