use crate::{cmd::Cmd, types::SaleId};
use serde::{Deserialize, Serialize};

const LIGHTNING_TOP_UP_PATH: &str = "lightning/top_up";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LightningTopUpInfo {
    pub id: String,
    pub invoice: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateLightningTopUp {
    pub months: u16,

    /// Sale ID for this price.
    ///
    /// If specified it *must* apply to this price or the request will be rejected.
    pub sale: Option<SaleId>,
}

impl Cmd for CreateLightningTopUp {
    type Output = LightningTopUpInfo;

    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = LIGHTNING_TOP_UP_PATH;
}
