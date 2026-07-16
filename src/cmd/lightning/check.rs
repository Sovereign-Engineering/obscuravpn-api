use crate::cmd::Cmd;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckLightningTopUpInfo {
    pub status: LightningTopUpStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum LightningTopUpStatus {
    Unpaid,
    Paid,
    Expired,
}

/// Check the status of a Lightning Top-up
///
/// Lightning top-ups are usually processed within seconds of payment.
///
/// Once the top-up indicates that it has been processed the account has already been credited with the payment.
///
/// Top-up records are eventually purged for privacy reasons. Expect this API to fail for old top-ups.
///
/// ## Expected Errors
/// - [`LightningTopUpNotFound`](crate::cmd::ApiErrorKind::LightningTopUpNotFound) if the `id` is invalid or pruned.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckLightningTopUp {
    pub id: String,
}

impl Cmd for CheckLightningTopUp {
    type Output = CheckLightningTopUpInfo;

    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "lightning/top_up/check";
}

#[test]
fn test_json() {
    let cmd_json = r#"
    {
        "id": "asdf"
    }
    "#;
    let output_json = r#"
    {
      "status": "paid"
    }
    "#;
    crate::cmd::check_cmd_json::<CheckLightningTopUp>(Some(cmd_json), Some(output_json));
}
