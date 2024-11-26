use crate::cmd::Cmd;
use crate::types::AccountInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetAccountInfo();

impl Cmd for GetAccountInfo {
    type Output = AccountInfo;
    const METHOD: http::Method = http::Method::GET;
    const PATH: &'static str = super::PATH;
}

#[test]
fn test_account_info_json() {
    let output_json = r#"
        {
          "id": "0000000000000000000",
          "active": true,
          "top_up": { "credit_expires_at": 1000 },
          "subscription": null
        }
    "#;
    crate::cmd::check_cmd_json::<GetAccountInfo>(None, Some(output_json));
}
