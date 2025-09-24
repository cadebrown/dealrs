//! An example agent strategy for Texas Holdem poker that is 'nitty', in that they will always check if possible, and fold against any bet.
//! 
//! This is a useful agent for testing and debugging, as well as for simple simulations and theoretical analysis. A few notable things about this agent:
//! 
//! * Their actions are completely deterministic, and independent of their hidden cards, shared cards, bank size, or any other state.
//! * They will fold pre-flop if they are not the big blind, and will check if they are the big blind.
//! * They will fold against any bet made.
//! 
//! This strategy is not very good, and it can be bluffed against to always win.
//! 

use crate::texas::agents::{TexasAgentAction, TexasAgentState};

use super::TexasAgent;

/// A 'nitty' agent that plays Texas Holdem poker, which means they will always check if possible, and fold against any bet.
/// 
/// Since the strategy does not depend on any hidden or shared state, it does not require memory to store any state.
#[derive(Debug)]
pub struct NittyTexasAgent { }

impl NittyTexasAgent {

    /// Create a new 'nitty' agent, which is a no-op constructor.
    pub fn new() -> Self {
        Self {}
    }
}

impl TexasAgent for NittyTexasAgent {

    /// Decision algorithm for a 'nitty' agent: given a certain bet size, they will always return 0. This indicates a check/fold decision, based on the current bet size.
    /// 
    /// If the current bet size is 0, this indicates a check. Otherwise, it indicates a fold.
    fn decide(&self, _: &TexasAgentState) -> TexasAgentAction {
        TexasAgentAction::new(0.into())
    }
}
