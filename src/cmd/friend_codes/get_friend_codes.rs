use std::collections::HashMap;

use crate::cmd::Cmd;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FriendCodeStatus {
    Claimed { code: String },
    Unclaimed { is_available: bool },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetFriendCodesOutput {
    pub friends: HashMap<String, FriendCodeStatus>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetFriendCodes {
    pub friends: Vec<String>,
}

impl Cmd for GetFriendCodes {
    type Output = GetFriendCodesOutput;

    const METHOD: http::Method = http::Method::GET;
    const PATH: &'static str = "friend_codes";
}
