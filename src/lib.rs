//! Rust toolkit for card games (like poker), providing card, deck, hand, and exploration utilities.
//! 
//! Also, includes higher level utilities for simulations, strategies, poker variants, and application development.
//! 

pub mod deck;
pub mod hand;

// TODO: separate this out?
pub mod combrs;

use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use rand_seeder::Seeder;

/// Construct a random number generator from a seed string, or use a default-initialized one if no seed is provided
pub fn rng_from_seed<S: AsRef<[u8]>>(seed: Option<S>) -> Box<dyn RngCore> {
    // TODO: allow for tradeoff between speed and quality of randomness? (ChaCha20 is great, but not neccessary for all use cases where large number of samples are needed)
    match seed {
        // with a seed, create a ChaCha20Rng from the seed
        Some(seed) => Box::new(Seeder::from(seed.as_ref()).into_rng::<ChaCha20Rng>()),
        // without a seed, use the default random number generator
        None => Box::new(rand::rng()),
    }
}
