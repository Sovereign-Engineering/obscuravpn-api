use crate::cmd::Cmd;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ApplePollSubscriptionOutput {}

/// This is used as a fallback to recover from discrepancies between the client
/// and server subscription state, i.e.:
/// - Setting the app account token after an offer code redemption.
/// - If the user made their first transaction during an outage.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ApplePollSubscription {
    // TODO: Rename this field if we ever bump the API version
    #[serde(rename = "transaction_id")]
    pub original_transaction_id: String,
}

impl Cmd for ApplePollSubscription {
    type Output = ApplePollSubscriptionOutput;

    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "apple/poll_subscription";
}
