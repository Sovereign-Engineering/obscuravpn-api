use super::{usize_from, WgFragmentHeaderData, WG_FRAGMENT_MESSAGE_HEADER_SIZE};
use bytes::Bytes;
use std::num::NonZeroU32;

pub enum ReassembleResult {
    /// Message was not fragmented, returned as-is.
    NotFragmented(Bytes),
    /// Message was reassembled from two fragments.
    Reassembled(Bytes),
    /// A single fragment was received with no matching counterpart; the max message size that caused fragmentation.
    UnmatchedFragment { max_message_size: u16 },
}

pub struct WgFragmentBuffer {
    max_fragment_size: u16,
    buffer: Vec<Option<Bytes>>,
}

impl WgFragmentBuffer {
    pub fn new(len: NonZeroU32, max_fragment_size: u16) -> Self {
        Self {
            max_fragment_size,
            buffer: vec![None; usize_from(len)],
        }
    }

    /// Process a potentially fragmented WG message.
    ///
    /// Returns the message if available, or the max message size from the fragment header if the fragment was buffered.
    ///
    /// Incorrect reassembly is theoretically possible if two fragmented messages are exactly `u32::MAX` or a multiple thereof apart, all intervening messages sharing the same buffer slot are lost, and only complementary halves of each survive. WireGuard's authenticated encryption will detect this downstream.
    pub fn reassemble(&mut self, new_message: Bytes) -> ReassembleResult {
        let Some(new_header) = WgFragmentHeaderData::from_message(&new_message) else {
            return ReassembleResult::NotFragmented(new_message);
        };
        let max_message_size = new_header.mtu;
        if new_message.len() > usize_from(self.max_fragment_size) {
            tracing::error!(
                message_id = "zg4h9td3",
                message_len = new_message.len(),
                max_fragment_size = self.max_fragment_size,
                "ignoring oversized WG message fragment"
            );
            return ReassembleResult::UnmatchedFragment { max_message_size };
        }
        let buffer_len = self.buffer.len();
        let buffer_element = &mut self.buffer[usize_from(new_header.message_idx) % buffer_len];
        match buffer_element {
            None => {
                *buffer_element = Some(new_message);
                ReassembleResult::UnmatchedFragment { max_message_size }
            }
            Some(old_message) => {
                let old_header = WgFragmentHeaderData::from_message(old_message).unwrap();
                if old_header.message_idx != new_header.message_idx {
                    *buffer_element = Some(new_message);
                    return ReassembleResult::UnmatchedFragment { max_message_size };
                }
                if old_header.second_fragment == new_header.second_fragment {
                    *buffer_element = Some(new_message);
                    return ReassembleResult::UnmatchedFragment { max_message_size };
                }
                let (first_msg, second_msg) = if old_header.second_fragment {
                    (new_message.as_ref(), old_message.as_ref())
                } else {
                    (old_message.as_ref(), new_message.as_ref())
                };
                let first_data = &first_msg[WG_FRAGMENT_MESSAGE_HEADER_SIZE..];
                let second_data = &second_msg[WG_FRAGMENT_MESSAGE_HEADER_SIZE..];
                let mut reassembled = Vec::with_capacity(first_data.len() + second_data.len());
                reassembled.extend_from_slice(first_data);
                reassembled.extend_from_slice(second_data);
                *buffer_element = None;
                ReassembleResult::Reassembled(reassembled.into())
            }
        }
    }
}
