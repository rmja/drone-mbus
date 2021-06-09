use crate::frameformat::FrameFormat;

const FIRST_BLOCK_PAYLOAD_SIZE: usize = 1 + 1 + 2 + 6;
const BLOCK_MAX_PAYLOAD_SIZES: [usize; 2 + 15] = [
    FIRST_BLOCK_PAYLOAD_SIZE,   // First block
    1 + 15,                     // Second block
    16,                         // Optional blocks
    16,
    16,
    16,
    16,
    16,
    16,
    16,
    16,
    16,
    16,
    16,
    16,
    16,
    6,
];
const MIN_PAYLOAD_SIZE: usize = FIRST_BLOCK_PAYLOAD_SIZE + 1;
const MAX_PAYLOAD_SIZE: usize = 256;
const MAX_FRAME_SIZE: usize = MAX_PAYLOAD_SIZE + 2 * BLOCK_MAX_PAYLOAD_SIZES.len();

pub struct FrameFormatA;

impl FrameFormat for FrameFormatA {
    fn block_has_crc(_block_index: usize) -> bool {
        true
    }

    fn block_max_payload_size(block_index: usize) -> usize {
        BLOCK_MAX_PAYLOAD_SIZES[block_index]
    }

    fn block_count_from_payload_size(mut payload_size: usize) -> Result<usize, ()> {
        if payload_size >= MIN_PAYLOAD_SIZE && payload_size <= MAX_PAYLOAD_SIZE {
            let mut block_count = 0;

            for block_max_payload_size in BLOCK_MAX_PAYLOAD_SIZES.iter() {
                block_count += 1;
                if payload_size > *block_max_payload_size {
                    // There are more blocks
                    payload_size -= block_max_payload_size;
                } else {
                    // This is the last, maybe not full, block
                    break;
                }
            }

            Ok(block_count)
        } else {
            Err(())
        }
    }

    fn block_count_from_frame_size(mut frame_size: usize) -> Result<usize, ()> {
        if frame_size >= MIN_PAYLOAD_SIZE + 2 * 2 && frame_size <= MAX_FRAME_SIZE {
            let mut block_count = 0;

            for block_max_payload_size in BLOCK_MAX_PAYLOAD_SIZES.iter() {
                block_count += 1;
                if frame_size > block_max_payload_size + 2 {
                    // There are more blocks
                    frame_size -= block_max_payload_size + 2;
                } else if frame_size > 2 {
                    // This is the last, maybe not full, block
                    break;
                }
                else {
                    // Invalid frame length
                    return Err(())
                }
            }

            Ok(block_count)
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
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, // 11..
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
            11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11,
            12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12,
            13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13,
            14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14,
            15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
            16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16,
            17, 17, 17, 17, 17, 17,                         // 251..=256
        ];

        for payload_size in 0..=MAX_PAYLOAD_SIZE {
            let expected = EXPECTED_BLOCK_COUNT[payload_size];
            if expected == 0 {
                assert!(FrameFormatA::block_count_from_payload_size(payload_size).is_err());
            }
            else {
                assert_eq!(expected, FrameFormatA::block_count_from_payload_size(payload_size).unwrap());
            }
        }

        assert!(FrameFormatA::block_count_from_payload_size(MAX_PAYLOAD_SIZE + 1).is_err());
    }

    #[test]
    pub fn can_get_block_count_from_frame_size() {
        const EXPECTED_BLOCK_COUNT: [usize; 1 + MAX_FRAME_SIZE] = [
            0,                                                      // 0
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,               // 1..=14 (CI field, i.e. second block, is required)
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0,   // 15..=32
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0, 0,   // 
            4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 0, 0,
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 0, 0,
            6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 0, 0,
            7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 0,
            8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 0, 0,
            9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 0, 0,
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 0, 0,
            11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 0, 0,
            12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 0, 0,
            13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 0, 0,
            14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 0, 0,
            15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 0, 0,
            16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 0, 0,
            17, 17, 17, 17, 17, 17,
        ];

        for frame_size in 0..=MAX_FRAME_SIZE {
            let expected = EXPECTED_BLOCK_COUNT[frame_size];
            println!("frame_size {:?}, expected block count {:?}", frame_size, expected);
            if expected == 0 {
                assert!(FrameFormatA::block_count_from_frame_size(frame_size).is_err());
            }
            else {
                assert_eq!(expected, FrameFormatA::block_count_from_frame_size(frame_size).unwrap());
            }
        }

        assert!(FrameFormatA::block_count_from_frame_size(MAX_FRAME_SIZE + 1).is_err());
    }

    #[test]
    pub fn frame_block_iter() {
        let mut iter = FrameFormatA::frame_block_iter(&[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0, 0,
            11, 0, 0,
        ]);
        assert_eq!(Some([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0, 0,].as_ref()), iter.next());
        assert_eq!(Some([11, 0, 0,].as_ref()), iter.next());
        assert_eq!(None, iter.next());
    }
}