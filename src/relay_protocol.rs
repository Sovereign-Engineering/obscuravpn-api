use static_assertions::const_assert;
use std::num::NonZeroU32;
use strum::{EnumIs, FromRepr};

pub const PROTOCOL_IDENTIFIER: u128 = 0x033d77afd0ded498c6ec25c75c5b0cf2;

#[derive(Debug, Copy, Clone, FromRepr, EnumIs)]
#[repr(u32)]
pub enum RelayOpCode {
    Ping = 0,
    Stop = 1,
    Token = 2,
    WireGuard = 3,
}

impl RelayOpCode {
    pub fn from_bytes(bytes: [u8; 4]) -> Option<Self> {
        Self::from_repr(u32::from_be_bytes(bytes))
    }
    pub fn to_bytes(self) -> [u8; 4] {
        (self as u32).to_be_bytes()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MessageContext(u32);

impl MessageContext {
    /// The smallest context ID for exchanges initiated by the client.
    pub const MIN_CLIENT_INITIATED: Self = Self(0);
    /// The smallest context ID for exchanges initiated by the relay.
    pub const MIN_RELAY_INITIATED: Self = Self(1);

    pub fn client_initiated(&self) -> bool {
        self.0 & 1 == 0
    }

    pub fn relay_initiated(&self) -> bool {
        !self.client_initiated()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MessageHeader {
    pub context_id: MessageContext,
    pub payload_length: u32,
}

impl MessageHeader {
    pub fn payload_length_usize(&self) -> usize {
        type PayloadLengthType = u32;
        let _: PayloadLengthType = self.payload_length;
        const_assert!(size_of::<PayloadLengthType>() <= size_of::<usize>());
        self.payload_length as usize
    }
}

impl From<MessageHeader> for [u8; 8] {
    fn from(header: MessageHeader) -> Self {
        let mut bytes = [0u8; 8];
        bytes[0..4].copy_from_slice(&header.context_id.0.to_be_bytes());
        bytes[4..8].copy_from_slice(&header.payload_length.to_be_bytes());
        bytes
    }
}

impl From<[u8; 8]> for MessageHeader {
    fn from(bytes: [u8; 8]) -> Self {
        Self {
            context_id: MessageContext(u32::from_be_bytes(bytes[0..4].try_into().unwrap())),
            payload_length: u32::from_be_bytes(bytes[4..8].try_into().unwrap()),
        }
    }
}

#[derive(Debug, Copy, Clone, EnumIs)]
#[repr(u32)]
pub enum RelayResponseCode {
    Ok = 0,
    Error(NonZeroU32),
}

impl RelayResponseCode {
    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        NonZeroU32::new(u32::from_be_bytes(bytes)).map(Self::Error).unwrap_or(Self::Ok)
    }
    pub fn to_bytes(self) -> [u8; 4] {
        match self {
            Self::Ok => [0, 0, 0, 0],
            Self::Error(n) => n.get().to_be_bytes(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MessageCode {
    Op(RelayOpCode),
    Response(RelayResponseCode),
}

impl MessageCode {
    pub fn from_bytes(bytes: [u8; 4], context_id: MessageContext, is_client: bool) -> Option<Self> {
        Some(match context_id.client_initiated() == is_client {
            true => MessageCode::Response(RelayResponseCode::from_bytes(bytes)),
            false => MessageCode::Op(RelayOpCode::from_bytes(bytes)?),
        })
    }
    pub fn to_bytes(self) -> [u8; 4] {
        match self {
            MessageCode::Op(op_code) => op_code.to_bytes(),
            MessageCode::Response(resp_code) => resp_code.to_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::relay_protocol::{MessageContext, MessageHeader};
    use rand::random;

    #[test]
    fn enc_dec() {
        for _ in 0..100 {
            let original_msg = MessageHeader {
                context_id: MessageContext(random()),
                payload_length: random(),
            };
            let encoded_msg: [u8; 8] = original_msg.into();
            let decoded_msg: MessageHeader = encoded_msg.into();
            assert_eq!(original_msg, decoded_msg);
        }
    }
}
