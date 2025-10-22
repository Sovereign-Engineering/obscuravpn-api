use crate::{
    cmd::{Cmd, MoneroTopUpId},
    types::SaleId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoneroTopUpInfo {
    pub id: MoneroTopUpId,
    pub pay_uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateMoneroTopUp {
    pub months: u16,
    pub sale: Option<SaleId>,
}

impl Cmd for CreateMoneroTopUp {
    type Output = MoneroTopUpInfo;

    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "monero/top_up";
}
