
extern crate dealrs;

use dealrs::{deck::CardMask, hand::{lutrank::LutRank, refhand5::RefHand5, Hand5, Rank5}};

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
    let engine_ref= RefHand5::new();

    // determine the best hand 
    let hand = engine_ref.hand5(cards);
    println!("hand_ref: {:}", hand);

    let engine_lut= LutRank::new();

    let rank = engine_lut.rank5(cards);
    println!("rank_lut: {:}", rank);
    let hand = engine_lut.hand5(cards);
    println!("hand_lut: {:}", hand);
    // println!("hand_lut: {:}", hand.unwrap());

    Ok(())
}
