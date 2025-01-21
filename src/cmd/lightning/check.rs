use crate::cmd::Cmd;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckLightningTopUpInfo {
    pub status: LightningTopUpStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum LightningTopUpStatus {
    Unpaid,
    Paid,
    Expired,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckLightningTopUp {
    pub id: String,
}

impl Cmd for CheckLightningTopUp {
    type Output = CheckLightningTopUpInfo;

    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "lightning/top_up/check";
}
