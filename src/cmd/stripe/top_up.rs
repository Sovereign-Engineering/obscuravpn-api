use crate::{cmd::Cmd, types::SaleId};
use serde::{Deserialize, Serialize};

const STRIPE_TOP_UP_PATH: &str = "stripe/top_up";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StripeTopUpInfo {
    pub payment_intent_client_secret: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateStripeTopUp {
    pub months: u16,

    /// Sale ID for this price.
    ///
    /// If specified it *must* apply to this price or the request will be rejected.
    pub sale: Option<SaleId>,
}

impl Cmd for CreateStripeTopUp {
    type Output = StripeTopUpInfo;

    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = STRIPE_TOP_UP_PATH;
}
