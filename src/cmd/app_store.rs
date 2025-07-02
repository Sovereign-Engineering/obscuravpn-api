use crate::{cmd::Cmd, types::AppStoreSubscriptionInfo};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppStoreCreateAppAccountToken;

impl Cmd for AppStoreCreateAppAccountToken {
    type Output = Uuid;

    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "app_store/app_account_token";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppStorePollSubscription {
    /// Any transaction ID known by the client for this account.
    ///
    /// Used as a fallback in case the server has missed all notifications for
    /// this account.
    pub transaction_id: Option<String>,
}

impl Cmd for AppStorePollSubscription {
    type Output = Option<AppStoreSubscriptionInfo>;

    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "app_store/poll_subscription";
}
