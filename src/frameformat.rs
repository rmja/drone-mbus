use core::{cmp::min, marker::PhantomData};

pub trait FrameFormat: Sized {
    fn block_has_crc(block_index: usize) -> bool;
    fn block_max_payload_size(block_index: usize) -> usize;
    fn block_max_frame_size(block_index: usize) -> usize {
        if Self::block_has_crc(block_index) {
            Self::block_max_payload_size(block_index) + 2
        }
        else {
            Self::block_max_payload_size(block_index)
        }
    }
    fn block_count_from_payload_size(payload_size: usize) -> Result<usize, ()>;
    fn block_count_from_frame_size(frame_size: usize) -> Result<usize, ()>;
    fn frame_block_iter<'a>(frame_bytes: &'a[u8]) -> FrameBlockIterator<'a, Self> {
        let block_count = Self::block_count_from_frame_size(frame_bytes.len()).unwrap();
        FrameBlockIterator {
            frame_bytes,
            block_count,
            block_index: 0,
            offset: 0,
            frame_format: PhantomData,
        }
    }
}

pub struct FrameBlockIterator<'a, FF: FrameFormat> {
    frame_bytes: &'a[u8],
    block_count: usize,
    block_index: usize,
    offset: usize,
    frame_format: PhantomData<FF>,
}

impl<'a, FF: FrameFormat> Iterator for FrameBlockIterator<'a, FF> {
    type Item = &'a[u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.block_index == self.block_count {
            None
        }
        else {
            let max_block_size = FF::block_max_frame_size(self.block_index);
            let block_size = min(self.frame_bytes.len() - self.offset, max_block_size);
            let result = Some(&self.frame_bytes[self.offset..self.offset + block_size]);
            self.block_index += 1;
            self.offset += block_size;
            result
        }
    }
}