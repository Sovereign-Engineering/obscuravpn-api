use serde::{Deserialize, Serialize};
use url::Url;

use crate::types::AccountId;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UrlOverride {
    pub api: String,
    pub web: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AcquireToken2Output {
    pub auth_token: String,
    pub url_override: Option<UrlOverride>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AcquireToken {
    pub account_id: AccountId,
}

impl AcquireToken {
    pub fn to_request2(&self, base_url: impl AsRef<str>) -> anyhow::Result<http::Request<String>> {
        let url = Url::parse(base_url.as_ref())?.join("token2")?;
        Ok(http::Request::post(url.as_str())
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(self)?)?)
    }
}

#[test]
fn test_json() {
    let json = r#"
    {
      "account_id": "0000000000000000000"
    }
    "#;
    check_json(json);
}

#[cfg(test)]
pub(crate) fn check_json(json: &str) {
    let login: AcquireToken = serde_json::from_str(json).unwrap();
    let json: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(json, serde_json::to_value(login).unwrap());
}
