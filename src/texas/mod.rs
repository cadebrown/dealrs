//! Game engine for Texas Holdem poker, as well as agentic interfaces for playing the game.
//! 
//! ## Game Rules
//! 
//! Texas Holdem consists of a table of players with associated stacks of chips (called their 'bank'), which must make forced bets (called 'blinds' and 'antes') at the beginning of each round. Each player is dealt 2 hidden cards (their 'hole' cards) at the start of each round, and can either check, call, raise, or fold. Players must call the maximum forced bet to stay in the hand, and can optionally raise.
//! 
//! There are then 3 stages of dealing community cards (called the 'shared' cards), which are visible and usable by all players. Each stage (the 'flop' with 3 cards, the 'turn' with 1 card, and the 'river' with 1 card) is followed by a round of betting, which can be checked (effectively, a bet of 0), or any of the other actions mentioned above.
//! 
//! At any point, if all but one player has folded, the remaining player wins the pot. Otherwise, the best hand using any 5 of the 7 cards (the hole cards and the shared cards) is determined, and the pot is awarded to the winner. For situations with ties, all-in players, and side pots, the rules are a bit more complex.
//! 
//! ## Environment Definition
//! 
//! An table environment for Texas Holdem is defined by its list of players, which are agents that can make decisions in particular scenarios. Each player has a unique ID, a position at the table, and a stack of chips (a 'currency').
//! 
//! ## Game Flow
//! 
//! Each game is independent, but consecutive games can affect the stacks of chips (obviously), and history of previous games can affect the decisions of players in subsequent games.
//! 
//! The basic flow of a single 'hand' or 'round' is as follows:
//! 
//! 1. Forced bets: The engine automatically creates a pot, and collects the blinds and antes from the players, depending on the game rules.
//! 2. Deal hole cards (2x): The engine deals 2 cards to each player, face down. In the digital world, this means that these are 'hidden' from the players when making decisions.
//!   * Optional betting: The engine iterates over players, prompting them to make a decision. By default, they must call the largest forced bet (normally called the 'big blind'). They can choose to check (only if they are the big blind), call, raise, or fold.
//! 4. Deal 'The Flop': The engine deals 3 cards to the table, face up. This event is visible to all agents.
//!   * Another round of optional betting, but now with no automatic forced bets. This means everyone has the ability to check if there are no bets made.
//! 5. Deal 'The Turn': The engine deals 1 card to the table, face up. This event is visible to all agents.
//!   * Another round of optional betting.
//! 6. Deal 'The River': The engine deals 1 card to the table, face up. This event is visible to all agents.
//!   * Another round of optional betting.
//! 7. Showdown: The engine reveals all the cards on the table, and the players' hole cards. The best 5-card hand is determined, and the pot is awarded to the winner.
//! 

pub mod agents;

pub mod cli;

use std::fmt::Debug;

use itertools::Itertools;
use rand::Rng;

use crate::{betting::{transfer_capped, Chips}, deck::{sample_cards, CardMask}, hand::{refhand5::RefHand5, Hand, Hand5}, texas::agents::{TexasAgent, TexasAgentState}};

/// Unique identifier for a player at the table
type PlayerID = u64;


/// The per-agent shared state of the game, which is visible to all agents at decision time
#[derive(Debug, Clone)]
pub struct TexasStateSharedAgent {

    /// Whether the agent is still in the hand, i.e. not folded
    active: bool,

    /// The size of the agent's stack of chips, or bankroll
    bank: Chips,

}

/// The public shared state of the game, which is visible to all agents and spectators at any time
#[derive(Debug, Clone)]
pub struct TexasStateShared {

    /// Collection of all agents at the table, and their individual shared state
    agents: Vec<TexasStateSharedAgent>,

    /// The current shared cards on the board
    shared: CardMask,

    /// The current pot size
    pot: Chips,

    /// The current bet size
    bet: Chips,

}

/// The private per-agent hidden state of the game, which is only visible to the agent at decision time
#[derive(Debug, Clone)]
pub struct TexasStateHidden {

    /// Your own hole cards, which are visible to you only
    hole: CardMask,

}

/// A game engine for Texas Holdem poker, which defines the rules of the game, drives the game flow, and manages the interactions between players
pub struct TexasEngine {

    /// Forced bet size for the 'ante', which is paid by each player before the first round of betting
    ante: Chips,

    /// List of all forced bet sizes, in order of payment from the button
    /// 
    /// For a 'standard structure' of just a small blind and big blind, this would be [small_blind, big_blind], which would by the players directly after the 'button' position. Betting would then begin with the player after the big blind.
    /// 
    /// In the case where there are more blinds than players, the blinds are 'wrapped' around the table, such that each player has to pay the sum of the blinds that apply to them.
    /// 
    /// For example, if there are 4 players, and the blinds are [1, 2, 3, 4, 5, 6], the first player after the button would pay (1+5=6) chips, the next player would pay (2+6=8) chips, the next player would pay (3) chips, and the last player (the button) would pay (4) chips.
    /// 
    /// In general, it is recommended that there are fewer blinds than players, but for consistency (or, in cases where players are removed from the game), this is a useful way to handle it. 
    /// 
    /// TODO: have a policy for 'capping' to the number of players?
    blinds: Vec<Chips>,

    /// List of all players at the table, which is just their agentic decision interface.
    /// 
    /// Right now, their IDs are just their index in this list.
    agents: Vec<Box<dyn TexasAgent>>,

    /// Keep track of the public state of the game, which is visible to all agents at any time
    shared: TexasStateShared,

    /// List of all private states of the game, which are the hole cards of all agents
    hiddens: Vec<TexasStateHidden>,

}

impl TexasEngine {
    /// Create a new Texas Holdem engine with the given betting structure
    /// 
    /// TODO: make a generic betting structure type, across game variants?
    pub fn new(ante: Chips, blinds: Vec<Chips>) -> Self {
        Self { ante, blinds, agents: Vec::new(), shared: TexasStateShared {
            agents: Vec::new(),
            shared: CardMask::NONE,
            pot: 0.into(),
            bet: 0.into(),
        }, hiddens: Vec::new() }
    }

    /// Add a new agent to the table, with the given bankroll
    /// 
    /// The agent is added to the end of the list
    pub fn add_agent(&mut self, agent: Box<dyn TexasAgent>, bank: Chips) {
        self.agents.push(agent);
        self.hiddens.push(TexasStateHidden { hole: CardMask::NONE });
        self.shared.agents.push(TexasStateSharedAgent { active: true, bank });
        self.shared.bet = bank;
    }

    /// Execute a single round of the game, which includes dealing hole cards, betting, and showdown
    pub fn execute<R: Rng>(&mut self, rng: &mut R) {

        // reset the game state
        self.shared.shared = CardMask::NONE;
        self.shared.pot = 0.into();

        // determine which agents can play this hand, i.e. they must have more than 0 chips
        for (id, _) in self.agents.iter_mut().enumerate() {
            self.shared.agents[id].active = if self.shared.agents[id].bank > 0.into() { true } else { false };

            // TODO: handle this better, somehow remove them from the pot early? or create a helper iterator for this...
            assert!(self.shared.agents[id].active, "Player {:} has 0 chips, which is not allowed", id);
        }

        // the 'button' position is the player with last action in normal direction of play
        // blinds are paid after the button, in increasing order
        let id_button = 0 as PlayerID;
        assert!(self.shared.agents[id_button as usize].active);

        // create a new deck of cards, starting out with all of them
        let mut deck = CardMask::FULL;

        // now, deal out hole cards to all agents
        for (id, hidden) in self.hiddens.iter_mut().enumerate() {
            if !self.shared.agents[id as usize].active {
                continue;
            }

            // sample 2 cards from the deck and store as hidden state
            hidden.hole = sample_cards(deck, 2, rng);

            // and remove them from the deck
            deck = deck & !hidden.hole;
        }
        
        // now, let's compute the forced bets to paid, per each player via their index
        // start off with the ante in each slot
        let mut forced = vec![self.ante; self.agents.len()];

        // then, iterate over the blinds, and collect them from each player in order, starting with the player after the button
        // also, consider wraparound as many times as needed and summing them up. this is critical for heads-up games, or when there are more blind structures than players (in this case, they are summed)
        for (idx, &bet) in self.blinds.iter().enumerate() {
            forced[(id_button as usize + idx + 1) % self.agents.len()] += bet;
        }

        // the ID of the player after the last forced bet
        let id_after_forced = ((id_button as usize + self.blinds.len() + 1) % self.agents.len()) as PlayerID;

        // the ID of the player after the button
        let id_after_button = ((id_button as usize + 1) % self.agents.len()) as PlayerID;

        // now, we do the first round of bets (called the 'pre-flop'), which includes forced bets
        // this will automatically withdraw those forced bets, and allow players to optionally raise, or just call the maximum forced bet
        // this will continue looping until all active players have called the maximum forced bet, or folded
        self.round_forced(id_after_forced, &forced);

        // deal the flop cards
        let cards_flop = sample_cards(deck, 3, rng);
        self.shared.shared = self.shared.shared | cards_flop;
        deck = deck & !cards_flop;

        // now, we do the first round of bets, which includes forced bets, but requires players to flat to stay in
        self.round(id_after_button);

        // deal the turn card
        let cards_turn = sample_cards(deck, 1, rng);
        self.shared.shared = self.shared.shared | cards_turn;
        deck = deck & !cards_turn;

        // now, we do the first round of bets, which includes forced bets, but requires players to flat to stay in
        self.round(id_after_button);

        // deal the river card
        let cards_river = sample_cards(deck, 1, rng);
        self.shared.shared = self.shared.shared | cards_river;
        deck = deck & !cards_river;

        // now, we do the first round of bets, which includes forced bets, but requires players to flat to stay in
        self.round(id_after_button);

        // now, do the showdown
        self.showdown();
    }


    /// Helper function to do the 'showdown' step, which should be ran at the end of a round
    /// 
    /// NOTE: do not call this directly, it is meant to be called by the engine after a round of betting
    pub fn showdown(&mut self) {
        let mut hands = vec![];
        println!("---- SHOWDOWN ----");
        let refbest5 = RefHand5::new();
        println!("pot: {:}", self.shared.pot);
        println!("shared: {:}", self.shared.shared);
        for (id, _) in self.agents.iter_mut().enumerate() {
            let hole = self.hiddens[id].hole;
            let hand = refbest5.hand5(hole | self.shared.shared);
            hands.push((id, hand));
            if self.shared.agents[id].active {
                println!("Player {:} has [hole: {:}] -> {:}", id, hole, hand);
            } else {
                println!("Player {:} was folded", id);
            }
        }

        // now, iterate over the hands in distinct order, i.e. first all the top winner(s), then the next best, etc respecting ties
        println!("WIN ORDER:");
        hands.sort_by(|a, b| a.1.cmp(&b.1));
        hands.reverse();

        let mut last_hand: Option<Hand> = None;
        let mut win_idx = 0;
        for (id, hand) in hands.iter() {
            if last_hand.is_none() || last_hand.unwrap() != *hand {
                last_hand = Some(*hand);
                win_idx += 1;
                println!("WIN GROUP {:}", win_idx);
            }
            println!("P{:} has {:}", id, hand);
        }
        
    }

    /// Helper function to run a round of betting, including some forced bets (such as blinds/antes).
    /// 
    /// Action will begin at the 'start' player, and continue until all active players have called the maximum forced bet, or the maximum raise, or folded.
    pub fn round_forced(&mut self, start: PlayerID, forced: &[Chips]) {
        println!("banks: {:}", self.shared.agents.iter().enumerate().map(|(id, bank)| format!("P{:} has {:}", id, bank.bank)).collect::<Vec<_>>().join(", "));

        // keep track of how much each player has paid
        let mut paid = vec![Chips::new(0); self.agents.len()];

        // first step is to collect the forced bets from the players
        // this is done so even if a player folds, they still pay the forced bets
        for (id, &bet) in forced.iter().enumerate() {
            // compute a capped transfer from the player's bankroll to the pot
            let (actual, _) = transfer_capped(&mut self.shared.pot, &mut self.shared.agents[id].bank, bet);

            // keep track of how much each player has paid
            paid[id] = actual;
        }

        // now, consider the maximum forced bet as what is required to be called to stay in the hand
        let mut ask_total = *forced.iter().max().unwrap();

        println!("ask_total: {:}", ask_total);

        let mut id_when_stop: Option<PlayerID> = None;

        // this is the iterator of the current player, which starts at the given location, and will wrap around as needed
        let mut id = start;

        // now, we start the monster loop that will continue until there is no more valid betting to be done in this round
        loop {

            // if we have a specific player that we are waiting for to stop, check if we have reached them
            if let Some(id_when_stop) = id_when_stop {
                if id == id_when_stop {
                    break;
                }
            }

            // get the player's active status, only consider them if they are still in the hand
            if self.shared.agents[id as usize].active {

                // get the player's bankroll, to determine if we even need to ask them to act
                if self.shared.agents[id as usize].bank > 0.into() {

                    // keep track of this specific ask
                    let ask = ask_total - paid[id as usize];

                    self.shared.bet = ask;
                    
                    // ask the agent to decide what they want to do
                    let mut amount = self.agents[id as usize].decide(&TexasAgentState { id, hidden: &self.hiddens[id as usize], shared: &self.shared }).amount;
                    // println!("RAW: P{:} decided to {:}", id, action);
                    
                    // now, sanitize the action to ensure it is valid
                    if amount > 0.into() && amount < ask {
                        println!("ERROR: P{:} tried to bet less than the current bet size of {:}, this will be considered a fold", id, ask);
                        amount = 0.into();
                    }

                    // TODO: sanitize to a minraise? or betting rules in particular?

                    // now, try to remove the chips from the agent
                    let (actual, allin) = transfer_capped(&mut self.shared.pot, &mut self.shared.agents[id as usize].bank, amount);
                    
                    // keep track of how much each player has paid
                    paid[id as usize] += actual;

                    if actual == 0.into() {
                        if ask > 0.into() {
                            println!("P{:}: FOLD", id);
                            self.shared.agents[id as usize].active = false;
                        } else {
                            println!("P{:}: CHECK", id);
                        }
                    } else if actual == ask {
                        if allin {
                            println!("P{:}: CALL {:} (ALL-IN)", id, actual);
                        } else {
                            println!("P{:}: CALL {:}", id, actual);
                        }
                    } else if actual > ask {
                        // on a raise, we will wait for the next loop to stop
                        id_when_stop = Some(id);
                        ask_total += actual - ask;
                        if allin {
                            println!("P{:}: RAISE {:} (ALL-IN)", id, actual);
                        } else {
                            println!("P{:}: RAISE {:}", id, actual);
                        }
                    } else {
                        unreachable!("unexpected case");
                    }
                }
            }

            // on the first iteration, just go ahead and mark the first player as the one to stop
            if let None = id_when_stop {
                id_when_stop = Some(id);
            }

            // advance to the next player, wrapping around if needed
            id = (id + 1) % self.agents.len() as PlayerID;
        }
    }

    /// Helper function to run a round of betting, with no forced bets
    pub fn round(&mut self, start: PlayerID) {
        // treat it as no-forced bets, i.e. all 0s
        let forced = vec![0.into(); self.agents.len()];

        // reuse the forced betting logic
        self.round_forced(start, &forced)
    }

}
