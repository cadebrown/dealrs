//! Betting policies and utilities for poker games, i.e. for chip value definitions, parsing, and sanitization.
//!
//! This is mainly used to abstract common betting operations from particular game variants, so they can be reused in different contexts. For instance, the game flow of Texas Holdem, Omaha, bomb-pots, multi-board variants, and so forth may vary, but it is essential to capture the betting logic so that they are all correctly implemented, and do not have subtle bugs.
//! 
//! 

use std::{fmt::Display, ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign}, str::FromStr};

/// Currency type for the game, which is a count of 'chip' values
// TODO: is this a u32/u64? or floats? I think integer is best, and stakes can be scaled up/down as needed (i.e. 100/200 stakes = $1/$2 with 0.01c increments)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Chips(u64);

impl Chips {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
}

/// Display the number of chips as a string, which is prefixed with a dollar sign ($) by default.
/// 
/// To display without the dollar sign, use the `{:#}` format specifier, which will emit just the number.
impl Display for Chips {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            // display without the dollar sign
            write!(f, "{:}", self.0)
        } else {
            // display with the dollar sign (default)
            write!(f, "${:}", self.0)
        }
    }
}

/// Automatic conversion from u64 to Chips, i.e. use it as the number of chips
impl From<u64> for Chips {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

/// Automatic conversion from Chips to u64, i.e. use it as the number of chips
impl From<Chips> for u64 {
    fn from(value: Chips) -> Self {
        value.0
    }
}

/// Automatic conversion from string to Chips, i.e. use it as the number of chips
/// This works on plain numbers, or numbers prefixed with a dollar sign ($)
impl FromStr for Chips {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('$') {
            // remove the dollar sign, and parse the rest as an integer
            Ok(Self(s[1..].parse::<u64>()?))
        } else {
            // parse the number as an integer directly
            Ok(Self(s.parse::<u64>()?))
        }
    }
}

impl Add<Chips> for Chips {
    type Output = Chips;
    fn add(self, other: Chips) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl AddAssign<Chips> for Chips {
    fn add_assign(&mut self, other: Chips) {
        *self = *self + other;
    }
}

impl Sub<Chips> for Chips {
    type Output = Chips;
    fn sub(self, other: Chips) -> Self::Output {
        Self(self.0 - other.0)
    }
}

impl SubAssign<Chips> for Chips {
    fn sub_assign(&mut self, other: Chips) {
        *self = *self - other;
    }
}

impl Mul<u64> for Chips {
    type Output = Chips;
    fn mul(self, other: u64) -> Self::Output {
        Self(self.0 * other)
    }
}

impl MulAssign<u64> for Chips {
    fn mul_assign(&mut self, other: u64) {
        *self = *self * other;
    }
}

/// Withdraw a requested amount or the maximum available from a source, returning the actual amount that was successfully withdrawn, and a boolean indicating if the source was depleted or not.
/// 
/// Most 'withdraw' operations (i.e. bank accounts), do NOT follow this policy (i.e. the entire operation will fail if the source is not enough). But, for poker, it is allowed to go 'all-in' and withdraw the full amount of the source. However, we need to know this to determine if the agent is 'all-in' or not, and to add to another source, or deposit into the pot.
pub fn withdraw_capped(from: &mut Chips, amount: Chips) -> (Chips, bool) {
    // to compute the actual amount we will be withdrawing, it is the minimum of the requested and available chip counts
    let actual = amount.min(*from);

    // then, actually withdraw the amount from the source
    *from -= actual;

    // and finally, 
    let depleted = *from == 0.into();
    (actual, depleted)
}


/// Deposit an amount into a source, adding it to the existing amount. This is just syntatic sugar for the addition operation, but can be more readable in some contexts.
pub fn deposit(to: &mut Chips, amount: Chips) {
    *to += amount;
}

/// A combined operation that first does a capped withdrawal from a source, and then deposits it into a destination.
/// 
/// Returns the actual amount that was successfully transferred, and a boolean indicating if the source was fully depleted or not.
pub fn transfer_capped(to: &mut Chips, from: &mut Chips, amount: Chips) -> (Chips, bool) {
    // first, do the withdrawal
    let (actual, depleted) = withdraw_capped(from, amount);

    // then, do the deposit
    deposit(to, actual);

    // finally, report the actual transfer amount
    (actual, depleted)
}
