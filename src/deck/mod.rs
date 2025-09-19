//! Defines the standard deck of cards, which is a collection of cards
//!
//! TODO: make this generic for other deck definitinos through traits? (low priority, but would be nice to have)
//! 
//! 

mod card;
mod cardmask;

pub use card::{Card, Rank, Suit};
pub use cardmask::{CardMask, RankMask, SuitMask};

use rand::Rng;

/// Randomly samples `num` cards from `src`, a set of available cards (with order, without replacement)
pub fn sample_cards_ordered<R: Rng>(src: CardMask, num: usize, rng: &mut R) -> Vec<Card> {
    assert!(src.count() >= num);
    let mut res = Vec::new();
    while res.len() < num {
        // generate a random card index
        let card_idx = rng.random_range(0..Card::NUM);

        // turn it into a proper card
        let card = Card::from_index(card_idx as u8);

        // check if the card is in the source mask (i.e. is available)
        if src.contains(card) && !res.contains(&card) {
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
    let mut res = CardMask::none();

    // TODO: smarter method that only calls the RNG exactly `n` times? (or 1?)
    // keep going until we have `n` cards
    while res.count() < num {
        // generate a random card index 
        let card_idx = rng.random_range(0..Card::NUM);

        // turn it into a proper card
        let card = Card::from_index(card_idx as u8);
        
        // add it to the result, which may do nothing if the card is not in the source mask
        // if so, it won't change the count and we'll keep going until we have `n` cards
        res = res.union(CardMask::from_single(card).intersect(src));
    }
    res
}
