use std::fmt::Debug;

// user interface agents for real human to play with
pub mod simplecli;

// simple reference implementation agents
pub mod fishy;
pub mod nitty;
pub mod shovy;

use crate::texas::{Chips, PlayerID, TexasStateHidden, TexasStateShared};


/// Represents the state visible to a single agent
#[derive(Debug, Clone)]
pub struct TexasAgentState<'a> {

    /// Your own unique identifier at the table
    pub id: PlayerID,

    /// Your own private state, which is visible to you only
    pub hidden: &'a TexasStateHidden,

    /// The public shared state of the game, which is visible to all agents and spectators
    pub shared: &'a TexasStateShared,

}



/// Represents a possible decision that an agent can make
/// 
/// Make a decision based on the given gamestate, which is how many chips to add to the pot.
/// 
/// This framing is generic, in that it can represent any action:
/// 
/// 1. Check: return 0 when there is no bet made
/// 2. Fold: return 0 when there is a bet made
/// 3. Bet: return 'num' chips when no bet was made
/// 4. Call: return 'num' chips when there is a bet made of exactly 'num' chips
/// 5. Raise: return 'num' chips when there is a bet made of less than 'num' chips
///
/// However, this must be validated to prevent cases where the agent returns 0 when they are actually required to bet, or if they return a larger amount than they have in their bankroll, or if there is a specific betting policy that must be followed (such as multiple of 5s, etc).
/// 
/// Right now, that causes a hard failure. But this is bad for interactive applications and user input. WIP
/// 
/// TODO: allow for 'intent' returns, i.e. TexasAgentAction::Check ? And hide this behind a 'strict' mode?
#[derive(Debug, Clone, Copy)]
pub struct TexasAgentAction {

    // How many chips to add to the pot, which could be a check, fold, bet, call, or raise
    pub amount: Chips,

}

impl TexasAgentAction {

    pub fn new(amount: Chips) -> Self {
        Self { amount }
    }
}

pub trait TexasAgent: Send + Sync + Debug{

    // fn decide(&self, shared: &TexasStateShared, hidden: &TexasStateHidden, ask: Chips) -> Chips;
    fn decide(&self, state: &TexasAgentState) -> TexasAgentAction;

}
