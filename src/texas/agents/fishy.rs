//! An example agent strategy for Texas Holdem poker that is 'fishy', in that they always call whatever is asked of them.
//! 
//! A 'fishy' agent will never fold, and never raise.
//! 
//! This is a useful agent for testing and debugging, as well as for simple simulations and theoretical analysis. A few notable things about this agent:
//! 
//! * Their actions are completely deterministic, and independent of their hidden cards, shared cards, bank size, or any other state.
//! * They have the unique property of being the 'cheapest' possible agent that always goes to showdown.
//!   * For the most 'expensive' agent that always goes to showdown, check out the [asdef](dealrs::texas::agents::shovy::ShovyTexasAgent) which always goes all-in at every opportunity.
//! * Their functional definition is extremely simple: given a certain bet size, they return exactly that amount.
//! 
//! Of course, this strategy is completely exploitable, and is not very good at playing the game.
//! 

use crate::texas::agents::{TexasAgentAction, TexasAgentState};

use super::TexasAgent;

/// A 'fishy' agent that plays Texas Holdem poker, which means they will always check if possible, call any bet made, never fold, and never raise.
/// 
/// Since the strategy does not depend on any hidden or shared state, it does not require memory to store any state.
#[derive(Debug)]
pub struct FishyTexasAgent { }

impl FishyTexasAgent {

    /// Create a new 'fishy' agent, which is a no-op constructor.
    pub fn new() -> Self {
        Self {}
    }
}

impl TexasAgent for FishyTexasAgent {

    /// Decision algorithm for a 'fish' agent: given a certain bet size, they will always return exactly that amount.
    /// 
    /// The action is sanitized by the game engine, which will cap it to the agent's bank size if it is greater.
    fn decide(&self, state: &TexasAgentState) -> TexasAgentAction {
        TexasAgentAction::new(state.shared.bet)
    }
}

