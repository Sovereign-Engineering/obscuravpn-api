use crate::cmd::Cmd;
use serde::{Deserialize, Serialize};

const LIGHTNING_TOP_UP_PATH: &str = "lightning/top_up";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LightningTopUpInfo {
    pub invoice: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateLightningTopUp {
    pub months: u16,
}

impl Cmd for CreateLightningTopUp {
    type Output = LightningTopUpInfo;

    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = LIGHTNING_TOP_UP_PATH;
}
