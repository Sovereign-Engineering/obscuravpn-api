use crate::cmd::{AuthRequired, Cmd};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppleCreateAppAccountTokenOutput {
    pub app_account_token: Uuid,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppleCreateAppAccountToken;

impl Cmd for AppleCreateAppAccountToken {
    type Output = AppleCreateAppAccountTokenOutput;
    type Auth = AuthRequired;

    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "apple/app_account_token";
}
