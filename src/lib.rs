//! A toolkit library for playing and developing card games (like poker and variants thereof), interactive applications, agentic strategies, and theoretical models like GTO solving.
//! 
//! If you just want to play poker, you can use these programs to get started instantly:
//! 
//! * [dealrs::texas::cli](texas::cli): a program that plays Texas Holdem poker with a few simple agents
//! 
//! If you want to develop your own application of a common game variant, you can use the high level modules to get started:
//! 
//! * [dealrs::texas](texas): a framework for working with the Texas Holdem poker variant, including game state management, agentic strategies, and 
//! 
//! In addition (unlike a lot of poker libraries), this crate includes modular lower level utility modules for building other game variants and conducting research:
//! 
//! * [dealrs::deck](deck): basic definitions of the standard deck of cards, and utilities for working with them
//! * [dealrs::hand](hand): standard hands of poker, and traits for working with them
//!   * [RefHand5](hand::refhand5::RefHand5): a reference implementation for determining the best 5-card hand from a given set of cards, which is pretty fast and used for verification
//! * [dealrs::betting](betting): common utilities for betting, chips, and sizing policies. These are generally game-agnostic, and thus can be reused in different contexts.
//!
//! Even lower level, there are exposed utilities for "pure math" behind the scenes. These modules are generally unstable and subject to change, so use them at your own risk:
//! 
//! * [dealrs::combrs](combrs): combinatorial utilities used for LUT generation, indexing, and decoding
//!


// TODO: separate this out into a separate crate?
pub mod combrs;

pub mod deck;
pub mod hand;

pub mod betting;

pub mod texas;

use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use rand_seeder::Seeder;

/// Construct a random number generator from a seed string, or use a default-initialized one if no seed is provided
/// 
/// TODO: allow a semantic clue for the RNG type? like fast, secure, entropy-seeded, etc?
pub fn rng_from_seed<S: AsRef<[u8]>>(seed: Option<S>) -> Box<dyn RngCore> {
    // TODO: allow for tradeoff between speed and quality of randomness? (ChaCha20 is great, but not neccessary for all use cases where large number of samples are needed)
    match seed {
        // with a seed, create a ChaCha20Rng from the seed
        Some(seed) => Box::new(Seeder::from(seed.as_ref()).into_rng::<ChaCha20Rng>()),
        // without a seed, use the default random number generator
        None => Box::new(rand::rng()),
    }
}
