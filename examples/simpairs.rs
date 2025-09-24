// examples/sim/pairs.rs - program that simulates pocket pairs in random deals of two cards

extern crate dealrs;

use dealrs::{deck::{sample_cards_ordered, CardMask}, rng_from_seed};

use kdam::{tqdm, BarExt};

use clap::Parser;

/// Simulation program that simulates pocket occurrences in random deals
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {

    /// Number of deals to simulate
    #[arg(short, long, default_value_t = 10000)]
    count: usize,

    /// Randomness seed string for deterministic generation
    #[arg(short, long)]
    seed: Option<String>,

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // construct a random number generator
    let mut rng = rng_from_seed(args.seed);

    // simulate the hands, and keep track of the number of pocket pairs
    let mut num_deals = 0;
    let mut num_pairs = 0;
    let mut pb = tqdm!(total = args.count);
    for _ in 0..args.count {

        // deal the two cards from the deck, and capture them
        let [c1, c2] = sample_cards_ordered(CardMask::FULL, 2, &mut rng)[..2].try_into().unwrap();

        // check whether the two cards are a pair (manually)
        let is_pair = c1.rank() == c2.rank();
        if is_pair {
            num_pairs += 1;
        }
        num_deals += 1;
        pb.update(1)?;
        pb.set_postfix(format!("pairs={:}/{:}={:.2}% (every {:.2} deals)", num_pairs, num_deals, num_pairs as f64 / num_deals as f64 * 100.0, num_deals as f64 / num_pairs as f64));
    }

    println!("Simulation complete, final results:");
    println!("Ran a total of {:} deals, finding {:} pairs", num_deals, num_pairs);
    println!("On a random deal, this is a {:.2}% chance of happening", num_pairs as f64 / num_deals as f64 * 100.0);
    println!("On average, you will find a pair every {:.2} deals", num_deals as f64 / num_pairs as f64);

    Ok(())
}
