use std::fmt;
use std::str::FromStr;

// a macro that counts the number of variants in a kind
macro_rules! count_variants {
    () => { 0 };
    ($head:ident $(, $tail:ident)*) => { 1 + count_variants!($($tail),*) };
}

// a macro that defines a 'kind', given their strings and values
macro_rules! make_kind {
    (
        $name:ident {
            $( $variant:ident => (value = $value:expr, text = $text:expr ) ),* $(,)?
        }
    ) => {
        #[repr(u8)]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
        pub enum $name {
            $( $variant = $value , )*
        }

        impl $name {
            pub const NUM: usize = count_variants!($( $variant ),*);
            pub const ALL: [Self; Self::NUM] = [
                $( Self::$variant , )*
            ];

            pub const fn from_index(index: u8) -> Self {
                Self::ALL[index as usize]
            }

            pub const fn to_index(self) -> u8 {
                self as u8
            }

            pub const fn text(self) -> &'static str {
                match self {
                    $( Self::$variant => $text , )*
                }
            }
        }
        
        impl FromStr for $name {
            type Err = &'static str;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                // attempt to match each variant
                for variant in Self::ALL {
                    if variant.text() == s {
                        return Ok(variant);
                    }
                }
                Err("invalid variant")
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.text())
            }
        }
    };
}

macro_rules! make_kind_combo {
    ( $name:ident = ($lhs_name:ident: $lhs_enum:ident) * ($rhs_name:ident: $rhs_enum:ident) ) => {
        // packed combination of two kinds into a single u8
        #[repr(transparent)]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
        pub struct $name(u8);

        impl $name {
            pub const NUM: usize = $lhs_enum::NUM * $rhs_enum::NUM;
            pub const ALL: [Self; Self::NUM] = {
                let mut all = [Self(0); Self::NUM];
                let mut i = 0;
                while i < Self::NUM {
                    all[i] = Self::from_index(i as u8);
                    i += 1;
                }
                all
            };
            
            pub const fn new(lhs: $lhs_enum, rhs: $rhs_enum) -> Self {
                Self::from_index(lhs.to_index() + rhs.to_index() * ($lhs_enum::NUM as u8))
            }
            
            pub const fn from_index(index: u8) -> Self {
                assert!(index < Self::NUM as u8, "index out of range");
                Self(index)
            }
            
            pub const fn to_index(self) -> u8 {
                self.0
            }

            pub const fn $lhs_name(self) -> $lhs_enum {
                $lhs_enum::from_index(self.0 % $lhs_enum::NUM as u8)
            }

            pub const fn $rhs_name(self) -> $rhs_enum {
                $rhs_enum::from_index(self.0 / $lhs_enum::NUM as u8)
            }
        }
        impl FromStr for $name {
            type Err = &'static str;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                if s.len() == 2 {
                // attempt to parse LHS and RHS
                let lhs = $lhs_enum::from_str(&s[..1])?;
                    // now, parse RHS from the rest of the string (skipping the first character)
                    let rhs = $rhs_enum::from_str(&s[1..])?;
                    Ok(Self::new(lhs, rhs))
                } else {
                    Err("invalid variant count")
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:}{:}", self.$lhs_name(), self.$rhs_name())
            }
        }
    };
}


make_kind! {
    Rank {
        Two      => ( value =  0, text = "2" ),
        Three    => ( value =  1, text = "3" ),
        Four     => ( value =  2, text = "4" ),
        Five     => ( value =  3, text = "5" ),
        Six      => ( value =  4, text = "6" ),
        Seven    => ( value =  5, text = "7" ),
        Eight    => ( value =  6, text = "8" ),
        Nine     => ( value =  7, text = "9" ),
        Ten      => ( value =  8, text = "T" ),
        Jack     => ( value =  9, text = "J" ),
        Queen    => ( value = 10, text = "Q" ),
        King     => ( value = 11, text = "K" ),
        Ace      => ( value = 12, text = "A" ),
    }
}

make_kind! {
    Suit {
        Spades   => ( value =  0, text = "s" ),
        Hearts   => ( value =  1, text = "h" ),
        Clubs    => ( value =  2, text = "c" ),
        Diamonds => ( value =  3, text = "d" ),
    }
}

make_kind_combo! {
    Card = (rank: Rank) * (suit: Suit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rank_roundtrip_all() {
        for rank in Rank::ALL {
            assert_eq!(rank, Rank::from_index(rank.to_index()));
        }
    }

    #[test]
    fn test_rank_roundtrip_index() {
        for i in 0..Rank::NUM as u8 {
            assert_eq!(i, Rank::from_index(i as u8).to_index());
        }
    }

    #[test]
    fn test_rank_roundtrip_text() {
        for rank in Rank::ALL {
            assert_eq!(rank, Rank::from_str(rank.text()).unwrap());
        }
    }

    #[test]
    fn test_rank_roundtrip_text_invalid() {
        assert!(Rank::from_str("invalid").is_err());
    }

    #[test]
    fn test_suit_roundtrip_all() {
        for suit in Suit::ALL {
            assert_eq!(suit, Suit::from_index(suit.to_index()));
        }
    }

    #[test]
    fn test_suit_roundtrip_index() {
        for i in 0..Suit::NUM as u8 {
            assert_eq!(i, Suit::from_index(i as u8).to_index());
        }
    }

    #[test]
    fn test_suit_roundtrip_text() {
        for suit in Suit::ALL {
            assert_eq!(suit, Suit::from_str(suit.text()).unwrap());
        }
    }

    #[test]
    fn test_suit_roundtrip_text_invalid() {
        assert!(Suit::from_str("invalid").is_err());
    }
    
    #[test]
    fn test_card_roundtrip_all() {
        for card in Card::ALL {
            assert_eq!(card, Card::from_index(card.to_index()));
        }
    }

    #[test]
    fn test_card_roundtrip_index() {
        for i in 0..Card::NUM as u8 {
            assert_eq!(i, Card::from_index(i as u8).to_index());
        }
    }

    #[test]
    fn test_card_roundtrip_text() {
        for card in Card::ALL {
            assert_eq!(card, Card::from_str(card.to_string().as_str()).unwrap());
        }
    }

    #[test]
    fn test_card_roundtrip_ranksuit() {
        for card in Card::ALL {
            assert_eq!(card, Card::new(card.rank(), card.suit()));
        }
    }

    #[test]
    fn test_card_roundtrip_text_invalid() {
        assert!(Card::from_str("invalid").is_err());
    }

    #[test]
    fn test_card_index_cases() {
        // keep special cases here in case something change as a canary test
        assert_eq!(Card::from_index(0), Card::from_str("2s").unwrap());
        assert_eq!(Card::from_index(1), Card::from_str("3s").unwrap());
        assert_eq!(Card::from_index(2), Card::from_str("4s").unwrap());
        assert_eq!(Card::from_index(3), Card::from_str("5s").unwrap());
        assert_eq!(Card::from_index(4), Card::from_str("6s").unwrap());
        assert_eq!(Card::from_index(5), Card::from_str("7s").unwrap());
        assert_eq!(Card::from_index(6), Card::from_str("8s").unwrap());
        assert_eq!(Card::from_index(7), Card::from_str("9s").unwrap());
        assert_eq!(Card::from_index(8), Card::from_str("Ts").unwrap());
        assert_eq!(Card::from_index(9), Card::from_str("Js").unwrap());
        assert_eq!(Card::from_index(10), Card::from_str("Qs").unwrap());
        assert_eq!(Card::from_index(11), Card::from_str("Ks").unwrap());
        assert_eq!(Card::from_index(12), Card::from_str("As").unwrap());
        assert_eq!(Card::from_index(13), Card::from_str("2h").unwrap());
        assert_eq!(Card::from_index(14), Card::from_str("3h").unwrap());
        assert_eq!(Card::from_index(15), Card::from_str("4h").unwrap());
        assert_eq!(Card::from_index(16), Card::from_str("5h").unwrap());
        assert_eq!(Card::from_index(17), Card::from_str("6h").unwrap());
        assert_eq!(Card::from_index(18), Card::from_str("7h").unwrap());
        assert_eq!(Card::from_index(19), Card::from_str("8h").unwrap());
        assert_eq!(Card::from_index(20), Card::from_str("9h").unwrap());
        assert_eq!(Card::from_index(21), Card::from_str("Th").unwrap());
        assert_eq!(Card::from_index(22), Card::from_str("Jh").unwrap());
        assert_eq!(Card::from_index(23), Card::from_str("Qh").unwrap());
        assert_eq!(Card::from_index(24), Card::from_str("Kh").unwrap());
        assert_eq!(Card::from_index(25), Card::from_str("Ah").unwrap());
        assert_eq!(Card::from_index(26), Card::from_str("2c").unwrap());
        assert_eq!(Card::from_index(27), Card::from_str("3c").unwrap());
        assert_eq!(Card::from_index(28), Card::from_str("4c").unwrap());
        assert_eq!(Card::from_index(29), Card::from_str("5c").unwrap());
        assert_eq!(Card::from_index(30), Card::from_str("6c").unwrap());
        assert_eq!(Card::from_index(31), Card::from_str("7c").unwrap());
        assert_eq!(Card::from_index(32), Card::from_str("8c").unwrap());
        assert_eq!(Card::from_index(33), Card::from_str("9c").unwrap());
        assert_eq!(Card::from_index(34), Card::from_str("Tc").unwrap());
        assert_eq!(Card::from_index(35), Card::from_str("Jc").unwrap());
        assert_eq!(Card::from_index(36), Card::from_str("Qc").unwrap());
        assert_eq!(Card::from_index(37), Card::from_str("Kc").unwrap());
        assert_eq!(Card::from_index(38), Card::from_str("Ac").unwrap());
        assert_eq!(Card::from_index(39), Card::from_str("2d").unwrap());
        assert_eq!(Card::from_index(40), Card::from_str("3d").unwrap());
        assert_eq!(Card::from_index(41), Card::from_str("4d").unwrap());
        assert_eq!(Card::from_index(42), Card::from_str("5d").unwrap());
        assert_eq!(Card::from_index(43), Card::from_str("6d").unwrap());
        assert_eq!(Card::from_index(44), Card::from_str("7d").unwrap());
        assert_eq!(Card::from_index(45), Card::from_str("8d").unwrap());
        assert_eq!(Card::from_index(46), Card::from_str("9d").unwrap());
        assert_eq!(Card::from_index(47), Card::from_str("Td").unwrap());
        assert_eq!(Card::from_index(48), Card::from_str("Jd").unwrap());
        assert_eq!(Card::from_index(49), Card::from_str("Qd").unwrap());
        assert_eq!(Card::from_index(50), Card::from_str("Kd").unwrap());
        assert_eq!(Card::from_index(51), Card::from_str("Ad").unwrap());
    }

}
