// refbest5.rs - example program that determines the best 5-card subset of a hand, using a reference implementation

extern crate dealrs;

use dealrs::{deck::CardMask, hand::{refbest5::RefBest5, Best5}};

use clap::Parser;

/// Simple program that deals cards from a random deck
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {

    /// Set of cards to evaluate
    #[arg(short, long)]
    cards: CardMask,

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // extract and print out what cards we have
    let cards = args.cards;
    println!("cards[{:?}]: {:}", cards.count(), cards);
    
    // create the reference evaluator
    let engine= RefBest5::new();

    // determine the best hand 
    let (used, hand) = engine.best5(cards);
    println!("used: {:}", used);
    println!("hand: {:}", hand);

    Ok(())
}
