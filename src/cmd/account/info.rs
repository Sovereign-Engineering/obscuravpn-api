use crate::cmd::Cmd;
use crate::types::AccountInfo;
use serde::{Deserialize, Serialize};

/// Get Account Info
///
/// ## Expected Errors
///
/// None
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetAccountInfo();

impl Cmd for GetAccountInfo {
    type Output = AccountInfo;
    const METHOD: http::Method = http::Method::GET;
    const PATH: &'static str = "account";
}

#[test]
fn test_account_info_json() {
    let output_json = r#"
        {
          "id": "0000000000000000000",
          "active": true,
          "top_up": {
            "active": true,
            "credit_expires_at": 1000
          },
          "stripe_subscription": null,
          "subscription": null,
          "apple_subscription": null,
          "google_subscription": null,
          "primary_funding": "top_up",
          "monero_pending_payments": [{
            "id": "asdas",
            "status": "confirming"
          }],
          "referral_code": "asdf",
          "has_referrer": true,
          "received_free_months": 2,
          "current_expiry": 1000,
          "auto_renews": null,
          "has_paid": true,
          "free_month_on_next_payment": true
        }
    "#;
    crate::cmd::check_cmd_json::<GetAccountInfo>(None, Some(output_json));
}
