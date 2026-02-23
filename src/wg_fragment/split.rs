use super::{WgFragmentHeaderData, WG_FRAGMENT_MESSAGE_HEADER_SIZE};
use bytes::Bytes;

#[derive(Debug, Default)]
pub struct WgMessageFragmenter {
    next_message_index: u32,
}

impl WgMessageFragmenter {
    pub fn fragment(&mut self, message: Bytes, max_message_size: u16) -> (Bytes, Option<Bytes>) {
        if message.len() <= usize::from(max_message_size) {
            return (message, None);
        }

        let message_idx = self.next_message_index;
        self.next_message_index = self.next_message_index.wrapping_add(1);

        let split_point = message.len() / 2;

        let first_header = WgFragmentHeaderData {
            message_idx,
            second_fragment: false,
            mtu: max_message_size,
        };
        let second_header = WgFragmentHeaderData {
            message_idx,
            second_fragment: true,
            mtu: max_message_size,
        };

        let mut first_frag = Vec::with_capacity(WG_FRAGMENT_MESSAGE_HEADER_SIZE + split_point);
        first_frag.extend_from_slice(&first_header.header_bytes());
        first_frag.extend_from_slice(&message[..split_point]);

        let remainder = message.len() - split_point;
        let mut second_frag = Vec::with_capacity(WG_FRAGMENT_MESSAGE_HEADER_SIZE + remainder);
        second_frag.extend_from_slice(&second_header.header_bytes());
        second_frag.extend_from_slice(&message[split_point..]);

        (first_frag.into(), Some(second_frag.into()))
    }
}
