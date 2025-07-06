use crate::cmd::Cmd;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ApplePollSubscriptionOutput {}

/// This is used as a fallback to recover from discrepancies between the client
/// and server subscription state, i.e. if the user made their first
/// transaction during an outage.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ApplePollSubscription {
    /// Any transaction ID known by the client for this account.
    pub transaction_id: String,
}

impl Cmd for ApplePollSubscription {
    type Output = ApplePollSubscriptionOutput;

    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "apple/poll_subscription";
}
