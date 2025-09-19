// deal.rs - example program that deals cards from a random deck, as a simple example

extern crate dealrs;

use dealrs::{deck::{sample_cards_ordered, CardMask}, rng_from_seed};

use clap::Parser;

/// Simple program that deals cards from a random deck
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {

    /// Number of cards to deal out
    #[arg(short, long, default_value_t = 1)]
    num: usize,

    /// Randomness seed string for deterministic generation
    #[arg(short, long)]
    seed: Option<String>,

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // construct a random number generator
    let mut rng = rng_from_seed(args.seed);

    // sample the cards, with order
    // it is normally more efficient to sample WITHOUT order, but this is done here so printing them out can be in any order
    let cards = sample_cards_ordered(CardMask::full(), args.num, &mut rng);
    assert_eq!(cards.len(), args.num, "expected {} cards, got {}", args.num, cards.len());

    // print out and display the cards
    for (idx, &card) in cards.iter().enumerate() {
        println!("card #{:?}: {:} (debug={:?}) (index={:?})", idx, card, card, card.to_index());
    }

    Ok(())
}
