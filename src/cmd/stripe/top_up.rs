use crate::cmd::Cmd;
use serde::{Deserialize, Serialize};

const STRIPE_TOP_UP_PATH: &str = "stripe/top_up";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StripeTopUpInfo {
    pub payment_intent_client_secret: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateStripeTopUp {
    pub months: u16,
}

impl Cmd for CreateStripeTopUp {
    type Output = StripeTopUpInfo;

    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = STRIPE_TOP_UP_PATH;
}
