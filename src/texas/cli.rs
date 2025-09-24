//! Program to play Texas Holdem in a command-line interface (CLI), such as a terminal.
//! 
//! This is the simplest way to just quickly play some poker, so run it and see what happens!
//! 
//! Examples:
//! 
//! ```shell
//! # default: play heads-up against a 'fishy' agent, with $100 bankroll, $1 ante and $2/$5 blinds
//! $ cargo run --bin texas --
//! 
//! # prints the help and usage information, describing all the options
//! $ cargo run --bin texas -- --help
//! ```
//! 

use clap::Parser;
use crate::{
    betting::Chips,
    rng_from_seed,
    texas::{
        agents::{
            fishy::FishyTexasAgent,
            nitty::NittyTexasAgent,
            shovy::ShovyTexasAgent,
            simplecli::SimpleCliTexasAgent,
            TexasAgent,
        },
        TexasEngine,
    },
};

/// Parse a string into a Texas agent implementation.
pub fn parse_agent(s: &str) -> Result<Box<dyn TexasAgent>, String> {
    match s {
        "simplecli" => Ok(Box::new(SimpleCliTexasAgent::new())),
        "fishy" => Ok(Box::new(FishyTexasAgent::new())),
        "nitty" => Ok(Box::new(NittyTexasAgent::new())),
        "shovy" => Ok(Box::new(ShovyTexasAgent::new())),
        _ => Err(format!("unknown player type: {s}")),
    }
}

/// Arguments for the Texas Holdem CLI, which determine the game configuration.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Starting bankroll for all players, in chips.
    // TODO: make this a list of bankrolls, for each player?
    #[arg(short, long, default_value = "100")]
    pub roll: Chips,

    /// Forced ante bet size, in chips.
    #[arg(short, long, default_value = "1")]
    pub ante: Chips,

    /// Forced blind bet sizes, in chips. These go in order of payment from the player after the button position.
    #[arg(short, long, value_delimiter = ',', default_value = "2,5")]
    pub blinds: Vec<Chips>,

    /// List of agentic players to sit in the game, in order of appearance at the table, starting with the initial button position.
    /// 
    /// NOTE: if you include multiple interactive players, you will be prompted for each of them (thus, you will be playing as multiple players). This can make the output somewhat confusing.
    // TODO: specify this and create a utility function for reuse in the agents module
    #[arg(short, long, value_delimiter = ',', default_value = "simplecli,fishy", num_args = 2..)]
    pub players: Vec<String>,

    /// Randomness seed string for deterministic generation.
    /// 
    /// If not provided, a default-initialized RNG will be used.
    #[arg(short, long)]
    pub seed: Option<String>,

}

impl Args {
    
    /// Run the Texas Holdem CLI with parsed arguments, useful as an entrypoint for the program.
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {

        // create the RNG from the provided seed (or, a default-initialized one if no seed is provided)
        let mut rng = rng_from_seed(self.seed);

        // create the game engine from the provided arguments, which includes the ante and blinds
        let mut eng = TexasEngine::new(self.ante, self.blinds);

        // add all the agents to the engine, with the provided bankroll
        for agent in self.players.iter().map(|s| parse_agent(s).unwrap()) {
            eng.add_agent(agent, self.roll);
        }

        // actually execute the game
        // TODO: error/return code?
        eng.execute(&mut rng);

        Ok(())
    }
}
