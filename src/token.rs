use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AcquireToken {
    pub account_id: String,
}

impl AcquireToken {
    pub fn to_request(&self, base_url: impl AsRef<str>) -> anyhow::Result<http::Request<String>> {
        let url = Url::parse(base_url.as_ref())?.join("token")?;
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
