//! A simplified command-line interface (CLI) agent for Texas Holdem poker, which prompts the user for interactive input on each decision.
//! 
//! If you want any nice-to-have features, please use the [`CliTexasAgent`] instead. That one is much nicer and more customizable. This one is aimed at being used by other programs not written in Rust, or esoteric systems.
//! 
//! This agent allows for input from a human running a program, or another program defined agent to make decisions over a textual interface, which will work in shells and terminals. It prints out useful context about the current state of the game, the history, as well as hidden and shared cards.
//! 
//! This agent, being the "simple" CLI version, is meant as a very understandable reference implementation for other agents to build upon. It is intentionally not customizable, and doesn't use any "fancy" features like box drawing, decorations, or cursor control in the terminal. This is so that it works on all platforms, and if a program is written in another language, it can easily parse the output, and produce input that is easy to parse.
//! 

use crate::texas::agents::{TexasAgentAction, TexasAgentState};

use super::TexasAgent;

/// A simplified CLI agent that prompts for input upon each decision, giving additional context about the history of the game.
#[derive(Debug)]
pub struct SimpleCliTexasAgent { }

impl SimpleCliTexasAgent {

    /// Create a new simplified CLI agent, which is a no-op constructor.
    pub fn new() -> Self {
        Self {}
    }
}

impl TexasAgent for SimpleCliTexasAgent {

    /// Decision algorithm for a 'simple' CLI agent: given a state, print it out to the console, and prompt the user for input.
    /// 
    /// The user input is sanitized by the game engine, which will ensure it is a valid bet size.
    fn decide(&self, state: &TexasAgentState) -> TexasAgentAction {
        println!("| Texas Agent CLI : decide()");
        println!("| context:");
        for (id, agent) in state.shared.agents.iter().enumerate() {
            let active = agent.active;
            let is_you_str = if id == state.id as usize { " (YOU)" } else { "" };
            let is_allin_str = if state.shared.agents[id as usize].bank == 0.into() { " [ALLIN]" } else { "" };
            if active {
                println!("| P{:} has {:}{:}{:}", id, state.shared.agents[id as usize].bank, is_allin_str, is_you_str);
            } else {
                println!("| P{:} is FOLDED{:}{:}", id, is_allin_str, is_you_str);
            }
        }
        println!("| pot: {:}", state.shared.pot);
        println!("| shared: {:}", state.shared.shared);
        println!("| hole: {:}", state.hidden.hole);
        println!("| enter the amount to call (={:}), raise (>{:}), or check/fold (=0):", state.shared.bet, state.shared.bet);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let out = input.trim().parse().unwrap();
        println!("| decision: {:}", out);
        TexasAgentAction::new(out)
    }
}
