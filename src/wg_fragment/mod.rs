use bytes::BufMut;

pub mod merge;
pub mod split;

// WireGuard uses message types 1-4. We picked 170, mid-range in the unassigned space, to avoid collisions with extensions that claim values near the boundaries.
const WG_FRAGMENT_MESSAGE_TYPE: u8 = 170;

const WG_FRAGMENT_MESSAGE_HEADER_SIZE: usize = 1 // message type
    + 1 // 1st (lowest) bit: fragment index; 2nd-8th bit: reserved
    + 2 // max message size (limit that caused fragmentation, little-endian)
    + 4; // message index (little-endian)

pub struct WgFragmentHeaderData {
    pub message_idx: u32,
    pub second_fragment: bool,
    pub mtu: u16,
}

impl WgFragmentHeaderData {
    pub fn header_bytes(&self) -> [u8; WG_FRAGMENT_MESSAGE_HEADER_SIZE] {
        let mut header = [0u8; WG_FRAGMENT_MESSAGE_HEADER_SIZE];
        let mut buf = &mut header[..];
        buf.put_u8(WG_FRAGMENT_MESSAGE_TYPE);
        buf.put_u8(u8::from(self.second_fragment));
        buf.put_u16_le(self.mtu);
        buf.put_u32_le(self.message_idx);
        header
    }
    pub fn from_message(message: &[u8]) -> Option<Self> {
        let (header, _data) = message.split_at_checked(WG_FRAGMENT_MESSAGE_HEADER_SIZE)?;
        let (message_type, header) = header.split_at(1);
        if message_type != [WG_FRAGMENT_MESSAGE_TYPE] {
            return None;
        }
        let (flags, header) = header.split_at(1);
        let second_fragment = (flags[0] & 1) != 0;
        let (mtu, header) = header.split_at(2);
        let mtu = u16::from_le_bytes(mtu.try_into().unwrap());
        let (message_idx, header) = header.split_at(4);
        let message_idx = u32::from_le_bytes(message_idx.try_into().unwrap());
        _ = header;

        Some(Self {
            message_idx,
            second_fragment,
            mtu,
        })
    }
}

fn usize_from(x: impl Into<u32>) -> usize {
    const _: () = assert!(usize::BITS >= 32, "usize smaller than u32");
    let x_u32: u32 = x.into();
    x_u32 as usize
}
