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
pub struct CheckFriendCodesOutput {
    pub friends: HashMap<String, FriendCodeStatus>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CheckFriendCodes {
    pub friends: Vec<String>,
}

impl Cmd for CheckFriendCodes {
    type Output = CheckFriendCodesOutput;

    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "friend_codes/status";
}
