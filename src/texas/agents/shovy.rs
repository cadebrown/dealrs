//! An example agent strategy for Texas Holdem poker that is 'shovy', in that they will go all-in at every opportunity.
//! 
//! This is a useful agent for testing and debugging, as well as for simple simulations and theoretical analysis. A few notable things about this agent:
//! 
//! * Their actions are completely deterministic, and independent of their hidden cards, shared cards, the bet size, or any other state.
//! * They have the unique property of being the 'most expensive' possible agent that always goes to showdown.
//!   * For the most 'cheap' agent that always goes to showdown, check out the [`FishyTexasAgent`] which always checks if possible, and calls any bet made.
//! * Their functional definition is extremely simple: given a certain bet size, they return their own bank size.
//! 
//! This strategy is terrible, of course, but is hard to always refute. For example, it goes all-in pre-flop if allowed to bet, so you must call off its entire bankroll each time. It is very easy for it to get lucky and win a hand (since you never have a 100% chance of winning pre-flop), so there is no guaranteed way to ever win against it. This is contrasted with the [`FishyTexasAgent`] which is completely exploitable, in that you check it down until the river, and if you have the nuts at that point, you go all-in and win.
//! 
//! The 'shovy' agent however, is harder to refute since it will begin gaining blinds and antes if you fold reasonable hands. If you wait for pocket aces (or, just a better-than-average hand) to play against it, even if you win it will likely have more money due to the blinds and antes.
//! 

use crate::texas::agents::{TexasAgentAction, TexasAgentState};

use super::TexasAgent;

/// A 'shovy' agent that plays Texas Holdem poker, which means they will always go all-in at every opportunity.
/// 
/// Since the strategy does not depend on any hidden or shared state, it does not require memory to store any state.
#[derive(Debug)]
pub struct ShovyTexasAgent { }

impl ShovyTexasAgent {

    /// Create a new 'shovy' agent, which is a no-op constructor.
    pub fn new() -> Self {
        Self {}
    }
}

impl TexasAgent for ShovyTexasAgent {

    /// Decision algorithm for a 'shovy' agent: given a certain bet size, they will always return their own bank size.
    /// 
    /// This will constitute an 'all-in' decision on every opportunity.
    /// 
    /// The action is sanitized by the game engine, which will cap it to the maximum bet size, if a limit is set.
    fn decide(&self, state: &TexasAgentState) -> TexasAgentAction {
        TexasAgentAction::new(state.shared.agents[state.id as usize].bank)
    }
}

