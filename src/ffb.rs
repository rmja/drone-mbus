use crate::frameformat::FrameFormat;

const FIRST_BLOCK_PAYLOAD_SIZE: usize = 1 + 1 + 2 + 6;
const BLOCK_MAX_PAYLOAD_SIZES: [usize; 3] = [
    FIRST_BLOCK_PAYLOAD_SIZE,   // First block
    1 + 115,                    // Second block
    130,                        // Optional block
];
const MIN_PAYLOAD_SIZE: usize = FIRST_BLOCK_PAYLOAD_SIZE + 1;
const MAX_PAYLOAD_SIZE: usize = 256;
const MAX_FRAME_SIZE: usize = MAX_PAYLOAD_SIZE + 2 * 2;

pub struct FrameFormatB;

impl FrameFormat for FrameFormatB {
    fn block_has_crc(block_index: usize) -> bool {
        block_index > 0
    }

    fn block_max_payload_size(block_index: usize) -> usize {
        BLOCK_MAX_PAYLOAD_SIZES[block_index]
    }

    fn block_count_from_payload_size(payload_size: usize) -> Result<usize, ()> {
        if payload_size >= MIN_PAYLOAD_SIZE && payload_size <= MAX_PAYLOAD_SIZE {
            if payload_size <= BLOCK_MAX_PAYLOAD_SIZES[0] + BLOCK_MAX_PAYLOAD_SIZES[1] {
                Ok(2)
            }
            else {
                Ok(3)
            }
        } else {
            Err(())
        }
    }

    fn block_count_from_frame_size(frame_size: usize) -> Result<usize, ()> {
        if frame_size >= MIN_PAYLOAD_SIZE + 2 && frame_size <= MAX_FRAME_SIZE {
            if frame_size <= BLOCK_MAX_PAYLOAD_SIZES[0] + BLOCK_MAX_PAYLOAD_SIZES[1] + 2 {
                Ok(2)
            }
            else if frame_size >= BLOCK_MAX_PAYLOAD_SIZES[0] + BLOCK_MAX_PAYLOAD_SIZES[1] + 2 + 2 {
                Ok(3)
            }
            else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}



#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn can_get_block_count_from_payload_size() {
        const EXPECTED_BLOCK_COUNT: [usize; 1 + MAX_PAYLOAD_SIZE] = [
            0,                                              // 0
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,                   // 1..= 10 (CI field, i.e. second block, is required)
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // 11..=126
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // 127..=260
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3,
        ];

        for payload_size in 0..=MAX_PAYLOAD_SIZE {
            let expected = EXPECTED_BLOCK_COUNT[payload_size];
            if expected == 0 {
                assert!(FrameFormatB::block_count_from_payload_size(payload_size).is_err());
            }
            else {
                assert_eq!(expected, FrameFormatB::block_count_from_payload_size(payload_size).unwrap());
            }
        }

        assert!(FrameFormatB::block_count_from_payload_size(MAX_PAYLOAD_SIZE + 1).is_err());
    }

    #[test]
    pub fn can_get_block_count_from_frame_size() {
        const EXPECTED_BLOCK_COUNT: [usize; 1 + MAX_FRAME_SIZE] = [
            0,                                                  // 0
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,                 // 1..=12 (CI field, i.e. second block, is required)
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,     // 13..=128
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2,
            0,                                                  // 129 is invalid
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,     // 130..=260
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3,
        ];

        for frame_size in 0..=MAX_FRAME_SIZE {
            let expected = EXPECTED_BLOCK_COUNT[frame_size];
            println!("frame_size {:?}, expected block count {:?}", frame_size, expected);
            if expected == 0 {
                assert!(FrameFormatB::block_count_from_frame_size(frame_size).is_err());
            }
            else {
                assert_eq!(expected, FrameFormatB::block_count_from_frame_size(frame_size).unwrap());
            }
        }

        assert!(FrameFormatB::block_count_from_frame_size(MAX_FRAME_SIZE + 1).is_err());
    }

    #[test]
    pub fn frame_block_iter() {
        let mut iter = FrameFormatB::frame_block_iter(&[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
            11, 0, 0,
        ]);
        assert_eq!(Some([1, 2, 3, 4, 5, 6, 7, 8, 9, 10,].as_ref()), iter.next());
        assert_eq!(Some([11, 0, 0,].as_ref()), iter.next());
        assert_eq!(None, iter.next());
    }
}