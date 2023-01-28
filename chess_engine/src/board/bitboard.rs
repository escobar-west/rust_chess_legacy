use std::{fmt, ops::BitAnd};
use crate::{
    squares::{Square, Square64},
    util::{
        Rank,
        File,
        SQUARE_120_TO_64,
        SQUARE_64_TO_120,
    },
    error::ChessError as Error,
};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;


// TODO: figure out if this is optimal for x86 or should be flipped
// LSB is A1, MSB H8
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BitBoard(pub u64);

// https://stackoverflow.com/questions/30680559/how-to-find-magic-bitboards
// TODO: generate own Magic Bitboard and implement
// const BIT_TABLE: [Square; 64] = [
//     Square::H8, Square::G4, Square::D1, Square::A5, Square::B4, Square::B6, Square::G3, Square::B5,
//     Square::H2, Square::C7, Square::C6, Square::F2, Square::D2, Square::F7, Square::D3, Square::C5,
//     Square::F8, Square::F4, Square::C1, Square::D7, Square::F3, Square::D6, Square::F6, Square::C2,
//     Square::C3, Square::H6, Square::B1, Square::G7, Square::B2, Square::B8, Square::A1, Square::D5,
//     Square::G8, Square::H4, Square::A6, Square::E1, Square::B7, Square::F1, Square::E7, Square::C4,
//     Square::E8, Square::G1, Square::H3, Square::E6, Square::G6, Square::D4, Square::A8, Square::A3,
//     Square::H1, Square::H5, Square::A7, Square::A4, Square::D8, Square::G2, Square::E2, Square::H7,
//     Square::G5, Square::E4, Square::C8, Square::E3, Square::F5, Square::B3, Square::E5, Square::A2
// ];

impl BitBoard {
    /// Counts number of set bits
    fn count_bits(&self) -> u8 {
        let mut count: u8 = 0;
        let mut b = self.0;
        while b > 0 {
            count += 1;
            // converts the current least significant 1 into 0111... with the -1 
            // then removes trailing 1s into 0s with the & (1000 & 0111 = 0000)
            b &= b - 1;
        }
        count
    }

    /// Sets the first set LSB to 0 and returns the index corresponding to it
    // NOTE: this is slow in comparison to magic bitboard implementation which
    // has a very real effect on performance of move generation and thus on bot ability
    fn pop_bit(&mut self) -> Option<u8> {
        let lsb_index: u8 = self.0.trailing_zeros() as u8;
        match lsb_index {
            // all zeros
            64 => { None },
            _ => {
                let mask: u64 = 1 << lsb_index;
                self.0 ^= mask;
                Some(lsb_index)
            }
        }
    }

    // TODO: implement magic bitboard version
    // // Relies on Magic BitBoard (see BIT_TABLE for more information)
    // fn pop_bit(&mut self) -> Square {
    //     let mut b = self.0 ^ (self.0 - 1);
    //     let fold = (b & 0xFF_FF_FF_FF) ^ (b >> 32);
    //     self.0 &= self.0 - 1;
    //     BIT_TABLE[((fold * 0x783a9b23) >> 26) as usize]
    // }

    /// Check if bit at index is set
    fn check_bit(&self, index: u8) -> Result<bool, Error> {
        match index > 63 {
            false => { Ok(self.0 & (1 << index) != 0) },
            _ => { Err(Error::BitBoardCheckBitInvalidIndex(index)) }
        }
    }

    /// Sets bit at index
    fn set_bit(&mut self, index: u8) -> Result<&u64, Error> {
        match index > 63 {
            false => {
                self.0 |= 1 << index;
                Ok(&self.0)
            },
            _ => { Err(Error::BitBoardSetBitInvalidIndex(index)) }
        }
    }

    /// Sets bit at index to 0
    fn unset_bit(&mut self, index: u8) -> Result<&u64, Error> {
        match index > 63 {
            false => {
                // XOR will toggle value at index so we should only call it
                // if the bit at index was already set
                if let Ok(true) = self.check_bit(index) {
                    self.0 ^= 1 << index;
                    Ok(&self.0)
                } else {
                    Err(Error::BitBoardUnsetNonSetBit(index))
                }
            },
            _ => { Err(Error::BitBoardUnsetBitInvalidIndex(index)) }
        }
    }
}


impl From<u64> for BitBoard {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

// TODO: had to explicitly do this despite implementing From for some reason
impl Into<u64> for BitBoard {
    fn into(self) -> u64 {
        self.0
    }
}

impl BitAnd for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in Rank::iter() {
            for file in File::iter() {
                let square_64 = Square64::from_file_and_rank(file, rank).expect("file and rank should be in range of 0..=7");
                // TODO: explore converting squares to bitboards and implementing bit operations
                match self.check_bit(square_64 as u8).expect("index should be less than or equal to 63") {
                    true => { write!(f, "1"); },
                    _ => { write!(f, "0"); },
                }
            }
            write!(f, "\n");
        }
        write!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitboard_display() {
        let input = BitBoard(0xFF00);
        let output = input.to_string();
        let expected = 
            "00000000\n11111111\n00000000\n00000000\n00000000\n00000000\n00000000\n00000000\n";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_count_bits_starting_white_pawn_position() {
        let input = BitBoard(0xFF00);
        let output = input.count_bits();
        let expected: u8 = 8;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_count_bits_empty() {
        let input = BitBoard(0);
        let output = input.count_bits();
        let expected: u8 = 0;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_check_set_bit_valid_index() {
        let index = 8;
        let input = BitBoard(0x00_00_00_00_00_00_01_00);
        let output = input.check_bit(index).unwrap();
        let expected = true;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_check_non_set_bit_valid_index() {
        let index = 8;
        let input = BitBoard(0x00_0F_00_00_00_00_00_00);
        let output = input.check_bit(index).unwrap();
        let expected = false;
        assert_eq!(output, expected);
    }

    #[should_panic]
    #[test]
    fn test_check_bit_invalid_index() {
        let index = 64;
        let input = BitBoard(0x00_00_00_00_00_00_01_00);
        let output = input.check_bit(index).unwrap();
    }

    #[test]
    fn test_set_bit_valid_index() {
        let index = 8;
        let mut input = BitBoard(0);
        let output = *input.set_bit(index).unwrap();
        let expected = 0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0001_0000_0000;
        assert_eq!(output, expected);
    }

    #[should_panic]
    #[test]
    fn test_set_bit_invalid_index() {
        let index = 64;
        let mut input = BitBoard(0);
        let output = *input.set_bit(index).unwrap();
    }

    #[test]
    fn test_unset_set_bit_valid_index() {
        let index = 8;
        let mut input = BitBoard(0x00_00_00_00_00_00_01_00);
        let output = *input.unset_bit(index).unwrap();
        let expected = 0;
        assert_eq!(output, expected);
    }

    #[should_panic]
    #[test]
    fn test_unset_non_set_bit_valid_index() {
        let index = 8;
        let mut input = BitBoard(0x00_F0_00_00_00_00_00_00);
        let output = *input.unset_bit(index).unwrap();
    }

    #[should_panic]
    #[test]
    fn test_unset_bit_invalid_index() {
        let index = 64;
        let mut input = BitBoard(0);
        let output = *input.unset_bit(index).unwrap();
    }

    #[test]
    fn test_pop_bit_single_set_bit() {
        let mut input = BitBoard(0x80_00_00_00_00_00_00_00);
        let output = input.pop_bit().unwrap();
        let expected_index: u8 = 63;
        let expected_board = BitBoard(0);
        assert_eq!(output, expected_index);
        assert_eq!(input, expected_board);
    }

    #[test]
    fn test_pop_bit_multiple_set_bit() {
        let mut input = BitBoard(0x0C_0F_00_D0_00_00_01_00);
        let output = input.pop_bit().unwrap();
        let expected_index: u8 = 8;
        let expected_board = BitBoard(0x0C_0F_00_D0_00_00_00_00);
        assert_eq!(output, expected_index);
        assert_eq!(input, expected_board);
    }

    #[should_panic]
    #[test]
    fn test_pop_bit_empty_board() {
        let mut input = BitBoard(0);
        let output = input.pop_bit().unwrap();
    }
}