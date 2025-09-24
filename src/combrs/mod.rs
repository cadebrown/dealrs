//! Combinatorial encoding and decoding utilities, such as binomial coefficients and multiset encoding/decoding to flattened index spaces
use num::{Integer, PrimInt};

pub mod bagspace;

/// Binomial coefficient C(n, k)
pub fn binom<T: PrimInt + Integer>(n: T, k: T) -> T {
    // check out-of-bounds cases, which are considered to be 0
    if k < T::zero() || k > n {
        return T::zero();
    }

    // now, compute it iteratively by multiplying and dividing
    let mut res = T::one();
    let mut i = T::zero();
    while i < k {
        res = res * (n - i) / (i + T::one());
        i = i + T::one();
    }
    res
}


// /// Encode a multiset (combination with repetition) into a unique, compact index space
// pub fn multiset_encode<I: PrimInt, O: PrimInt>(items: &[I]) -> O {

//     // keep track of the index while we are calculating it as the sum of binomial coefficients, which guarantees uniqueness and compactness
//     let mut idx = 0usize;

//     // now, iterate over the items 
//     for (i, a) in items.iter().map(|x| x.to_usize().unwrap()).enumerate() {
//         // compute the binomial coefficient and add it to the index
//         idx = idx + binom(a + i, i + 1);
//     }
//     O::from::<usize>(idx).unwrap()
// }


// /// Decode an index into a multiset of length k with values in 0..n
// pub fn multiset_decode<I: PrimInt, O: PrimInt>(idx: I, n: O, k: O, items: &mut [O]) {

//     // basically, we are just inverting the encoding process, so start with the index and work backwards
//     let mut idx = idx.to_usize().unwrap();

//     // convert to usize for the math
//     let n = n.to_usize().unwrap();
//     let k = k.to_usize().unwrap();

//     // same as above, we start with 'a' as the largest possible item, and work backwards towards a=0
//     let mut a = n + k - 1usize;
//     // the position of the item we are currently decoding, which starts at the end and moves backwards
//     // this indicates which index into 'items' we are currently writing to
//     let mut pos = k - 1;

//     // start with the largest possible item, and work backwards, since we greedily choose the largest possible item each time
//     for i in (1..=k).rev() {
//         // find largest a such that C(x, i) <= idx, which tells us what size chunk to 'chop off' from the index value
//         while binom(a, i) > idx {
//             a -= 1;
//         }

//         // write the item to the current position
//         items[pos] = O::from::<usize>(a - pos).unwrap();

//         // we need to break out early if we are at the first item, so we don't underflow values
//         if i == 1 {
//             break;
//         }

//         // otherwise, on most loops, we need to record the difference and 'chop off' that chunk from the index, and keep track of the new value of 'a'
//         idx -= binom(a, i);
//         a -= 1;
//         pos -= 1;
//     }
// }

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    // #[test]
    // fn test_multiset_encdec() {

    //     // first, iterate over all possible sequences and make sure they can be encoded and decoded back to the original sequence
    //     let n = 13;
    //     let k = 5;
    //     for seq in (0..n).combinations_with_replacement(k) {
    //         let mut seq = seq.iter().map(|x| *x as usize).collect::<Vec<usize>>();
    //         seq.sort();
    //         let idx: usize = multiset_encode(&seq);
    //         let mut decoded = vec![0; k];
    //         multiset_decode(idx, n, k, &mut decoded);
    //         println!("seq: {:?} => {:?} => {:?}", seq, idx, decoded);
    //         assert_eq!(seq, decoded);
    //     }

    //     // now, iterate over all possible indices and make sure they can be decoded back to the original sequence
    //     for idx in 0..binom(n+k-1, k) {
    //         let mut seq = vec![0; k];
    //         multiset_decode(idx, n, k, &mut seq);
    //         let idx_redo: usize = multiset_encode(&seq);
    //         println!("idx: {:?} => {:?} => {:?}", idx, seq, idx_redo);
    //         assert_eq!(idx, idx_redo);
    //     }

    // }
}






