use bitvec::prelude::*;
use alloc::vec::Vec;

pub struct ThreeOutOfSix;

// Table 10 in EN13757-4
const ENCODE_TABLE: [u8; 0x10] = [22, 13, 14, 11, 28, 25, 26, 19, 44, 37, 38, 35, 52, 49, 50, 41,];
const DECODE_TABLE: [i8; 0x40] = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,  3, -1,  1,  2, -1,
    -1, -1, -1,  7, -1, -1,  0, -1, -1,  5,  6, -1,  4, -1, -1, -1,
    -1, -1, -1, 11, -1,  9, 10, -1, -1, 15, -1, -1,  8, -1, -1, -1,
    -1, 13, 14, -1, 12, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
];

impl ThreeOutOfSix {
    pub fn encode(data: &[u8]) -> BitVec<u8, Msb0> {
        #[derive(PartialEq)]
        enum State { A, B, C, D, }

        let bits = data.len() * 12; // * 8 / 4 * 6
        let mut result = Vec::with_capacity((bits + 8 - 1) / 8);
        let mut state = State::A;
        let mut carry = 0;

        for nibble_index in 0..2*data.len() {
            let byte = data[nibble_index/2];
            let symbol = if nibble_index & 1 == 0 {
                ENCODE_TABLE[(byte >> 4) as usize]
            }
            else {
                ENCODE_TABLE[(byte & 0x0F) as usize]
            };

            match state {
                State::A => {
                    // AAAAAAXX
                    carry = symbol << 2;
                    state = State::B;
                },
                State::B => {
                    // XXXXXXBB
                    result.push(carry | (symbol >> 4));
                    // BBBBXXXX
                    carry = symbol << 4;
                    state = State::C;
                },
                State::C => {
                    // XXXXCCCC
                    result.push(carry | (symbol >> 2));
                    // CCXXXXXX
                    carry = symbol << 6;
                    state = State::D;
                },
                State::D => {
                    // XXDDDDDD
                    result.push(carry | symbol);
                    state = State::A;
                },
            }
        }

        if state != State::A {
            // Ensure that carry is fully written
            result.push(carry);
        }

        let mut result = BitVec::from_vec(result);
        result.resize(bits, false);
        result
    }

    pub fn decode(encoded: &BitVec<u8, Msb0>) -> Result<Vec<u8>, ()> {
        let data_len = encoded.len() / 12; // / 6 / 2
        if encoded.len() != data_len * 12 {
            // Must decode a multiple of bytes
            return Err(());
        }
        let mut result = Vec::with_capacity(data_len);
        let mut carry = -1;

        for symbol_slice in encoded.chunks_exact(6) {
            let symbol =
                ((symbol_slice[0] as u8) << 5) +
                ((symbol_slice[1] as u8) << 4) +
                ((symbol_slice[2] as u8) << 3) +
                ((symbol_slice[3] as u8) << 2) +
                ((symbol_slice[4] as u8) << 1) +
                ((symbol_slice[5] as u8) << 0);
            let nibble = DECODE_TABLE[symbol as usize];

            if nibble == -1 {
                return Err(());
            }
            else {
                if carry == -1 {
                    carry = nibble;
                }
                else {
                    result.push((carry as u8) << 4 | nibble as u8);
                    carry = -1;
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn can_encode_example() {
        let data = vec![0x2F, 0x44, 0x68, 0x50, 0x27, 0x21, 0x45, 0x30, 0x50, 0x62, 0xBD, 0xCC, 0xA2, 0x06, 0x9F, 0x1B, 0x11, 0x06, 0xC0, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x55, 0xA3, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF];
        let encoded = ThreeOutOfSix::encode(&data);
        let encoded_expected: Vec<u8> = vec![
            0x3a, 0x97, 0x1c, 0x6a, 0xc6, 0x56, 0x39, 0x33,
            0x8d, 0x71, 0x92, 0xd6, 0x65, 0x66, 0x8e, 0x8f,
            0x1d, 0x34, 0x98, 0xe5, 0x9a, 0x96, 0x93, 0x63,
            0x34, 0xd5, 0x9a, 0xd1, 0x63, 0x56, 0x59, 0x65,
            0x96, 0x59, 0x65, 0x96, 0x59, 0x65, 0x96, 0x59,
            0x65, 0x96, 0x65, 0x99, 0x8b, 0x59, 0x65, 0x96,
            0x59, 0x65, 0x96, 0x59, 0x65, 0x96, 0x59, 0x65,
            0x96, 0x59, 0x65, 0x96, 0x59, 0x65, 0x96, 0x59,
            0x65, 0x96, 0x59, 0x65, 0x96, 0xa6, 0x9a, 0x69,
            0x59, 0x65, 0x96, 0x59, 0x65, 0x96, 0x59, 0x65,
            0x96, 0xa6, 0x9a, 0x69,
        ];
        let expected: BitVec<u8, Msb0> = BitVec::from_vec(encoded_expected);

        assert_eq!(expected, encoded);
    }
    
    #[test]
    pub fn can_encode_correctly_terminates() {
        let data: [u8; 1] = [0x12];
        let encoded = ThreeOutOfSix::encode(&data);

        assert_eq!(bitvec![
            0,0,1,1,0,1,
            0,0,1,1,1,0], encoded);
    }

    #[test]
    pub fn can_decode() {
        let data = vec![0x2F, 0x44, 0x68, 0x50, 0x27, 0x21, 0x45, 0x30, 0x50, 0x62, 0xBD, 0xCC, 0xA2, 0x06, 0x9F, 0x1B, 0x11, 0x06, 0xC0, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x55, 0xA3, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF];
        let encoded = ThreeOutOfSix::encode(&data);
        let decoded = ThreeOutOfSix::decode(&encoded);
        assert_eq!(data, decoded.unwrap());
    }
}