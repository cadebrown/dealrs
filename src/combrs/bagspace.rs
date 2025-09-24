//! Utilities for working with bags (a.k.a. multisets), and the spaces defined by various constraints on them

use num::PrimInt;
use serde::{Deserialize, Serialize};

use crate::combrs::binom;

/// A helper structure to encode and decode sets (a.k.a. combinations) into and from index spaces, as well as iteration and construction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct SetSpace {

    /// The number of items to choose from, i.e. the universe of possible single items
    pub num_uni: usize,

}

impl SetSpace {
    pub fn new(num_uni: usize) -> Self {
        Self { num_uni }
    }

    /// Encode a sequence of items into single index within this space
    pub fn enc<I: PrimInt, O: PrimInt>(&self, seq: &[I]) -> O {
        assert!(seq.len() == self.num_uni, "expected sequence to be complete");

        // keep track of the index while we are calculating it as the sum of binomial coefficients, which guarantees uniqueness and compactness
        let mut idx = 0usize;

        // now, iterate over the items 
        for (i, a) in seq.iter().map(|x| x.to_usize().unwrap()).enumerate() {
            // compute the binomial coefficient and add it to the index
            idx = idx + (1usize << a);
        }
        O::from::<usize>(idx).unwrap()
    }

    /// Decode an index into a multiset 
    pub fn dec<I: PrimInt, O: PrimInt>(&self, idx: I, items: &mut [O]) {

        assert!(items.len() == self.num_uni, "expected sequence to be complete");
        
        // basically, we are just inverting the encoding process, so start with the index and work backwards
        let mut idx = idx.to_usize().unwrap();

        // start with the largest possible item, and work backwards, since we greedily choose the largest possible item each time
        for i in (1..=self.num_uni).rev() {
            // find largest a such that 2^a <= idx, which tells us what size chunk to 'chop off' from the index value
            if (1usize << i) & idx == 0 {
                // write the item to the current position
                items[i] = O::from::<usize>(i).unwrap();
            }
            idx -= 1usize << i;
        }
    }


}



/// A helper structure to encode and decode bags (a.k.a. multisets) into and from index spaces, as well as iteration and construction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct BagSpace {

    /// The number of items to choose from, i.e. the universe of possible single items
    pub num_uni: usize,

    /// The number of items in a particular bag within this space, i.e. the number to choose
    pub num_seq: usize,

}


impl BagSpace {
    pub fn new(num_uni: usize, num_seq: usize) -> Self {
        Self { num_uni, num_seq }
    }

    /// Encode a sequence of items into single index within this space
    pub fn enc<I: PrimInt, O: PrimInt>(&self, seq: &[I]) -> O {
        assert!(seq.len() == self.num_seq, "expected sequence to be complete");

        // keep track of the index while we are calculating it as the sum of binomial coefficients, which guarantees uniqueness and compactness
        let mut idx = 0usize;

        // now, iterate over the items 
        for (i, a) in seq.iter().map(|x| x.to_usize().unwrap()).enumerate() {
            // compute the binomial coefficient and add it to the index
            idx = idx + binom(a + i, i + 1);
        }
        O::from::<usize>(idx).unwrap()
    }

    /// Decode an index into a multiset 
    pub fn dec<I: PrimInt, O: PrimInt>(&self, idx: I, items: &mut [O]) {

        assert!(items.len() == self.num_seq, "expected sequence to be complete");
        
        // basically, we are just inverting the encoding process, so start with the index and work backwards
        let mut idx = idx.to_usize().unwrap();

        // same as above, we start with 'a' as the largest possible item, and work backwards towards a=0
        let mut a = self.num_uni + self.num_seq - 1usize;
        // the position of the item we are currently decoding, which starts at the end and moves backwards
        // this indicates which index into 'items' we are currently writing to
        let mut pos = self.num_seq - 1;

        // start with the largest possible item, and work backwards, since we greedily choose the largest possible item each time
        for i in (1..=self.num_seq).rev() {
            // find largest a such that C(x, i) <= idx, which tells us what size chunk to 'chop off' from the index value
            while binom(a, i) > idx {
                a -= 1;
            }

            // write the item to the current position
            items[pos] = O::from::<usize>(a - pos).unwrap();

            // we need to break out early if we are at the first item, so we don't underflow values
            if i == 1 {
                break;
            }

            // otherwise, on most loops, we need to record the difference and 'chop off' that chunk from the index, and keep track of the new value of 'a'
            idx -= binom(a, i);
            a -= 1;
            pos -= 1;
        }
    }


}
