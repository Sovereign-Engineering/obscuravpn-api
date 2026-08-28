use crate::cmd::Cmd;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClaimFriendCodeOutput {
    pub code: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClaimFriendCode {
    pub friend: String,
}

impl Cmd for ClaimFriendCode {
    type Output = ClaimFriendCodeOutput;

    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "friend_codes";
}
