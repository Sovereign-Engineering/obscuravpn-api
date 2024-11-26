use crate::cmd::Cmd;
use serde::{Deserialize, Serialize};

const CREATE_STRIPE_SUBSCRIPTION_CHECKOUT_PATH: &str = "stripe/create_checkout_session";
const CREATE_PORTAL_SESSION_CHECKOUT_PATH: &str = "stripe/create_portal_session";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateStripeSubscriptionCheckout {}

impl Default for CreateStripeSubscriptionCheckout {
    fn default() -> Self {
        Self::new()
    }
}

impl CreateStripeSubscriptionCheckout {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateStripeSubscriptionCheckoutOutput {
    pub checkout_url: String,
}

impl CreateStripeSubscriptionCheckoutOutput {
    pub fn new(checkout_url: String) -> Self {
        Self { checkout_url }
    }
}

impl Cmd for CreateStripeSubscriptionCheckout {
    type Output = CreateStripeSubscriptionCheckoutOutput;
    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = CREATE_STRIPE_SUBSCRIPTION_CHECKOUT_PATH;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateStripeManageSubscriptionSession {
    pub session_id: String,
}

impl CreateStripeManageSubscriptionSession {
    pub fn new(session_id: String) -> Self {
        Self { session_id }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateStripeManageSubscriptionSessionOutput {
    pub portal_url: String,
}

impl CreateStripeManageSubscriptionSessionOutput {
    pub fn new(portal_url: String) -> Self {
        Self { portal_url }
    }
}

impl Cmd for CreateStripeManageSubscriptionSession {
    type Output = CreateStripeManageSubscriptionSessionOutput;

    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = CREATE_PORTAL_SESSION_CHECKOUT_PATH;
}
