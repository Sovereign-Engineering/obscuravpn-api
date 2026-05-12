use crate::cmd::Cmd;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GoogleBillingDetails {
    pub promo_code: Option<String>,
}

impl Cmd for GoogleBillingDetails {
    type Output = GoogleBillingDetailsOutput;

    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "google/billing_details";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GoogleBillingDetailsOutput {
    pub product_id: String,
    pub base_plan_id: String,
    pub offer_id: Option<String>,
}
