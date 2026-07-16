use crate::pow::PowOutput;
use crate::types::AccountId;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UrlOverride {
    pub api: String,
    pub web: String,
}

/// Response of auth request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AcquireToken2Output {
    /// The auth token.
    ///
    /// The token is used by adding an `Authorization: Bearer {token}` header to your requests.
    ///
    /// The token has no definite expiry date. It is recommended to cache tokens indefinitely, only acquiring a new one when the API returns a [`MissingOrInvalidAuthToken`](crate::cmd::ApiErrorKind::MissingOrInvalidAuthToken) error.
    pub auth_token: String,

    /// Internal use.
    pub url_override: Option<UrlOverride>,
}

/// Acquire an auth token.
///
/// This body should be sent as the JSON body of a `POST` request to the `/api/token2` endpoint.
///
/// The response is an [`AcquireToken2Output`].
///
/// ## Expected Errors
///
/// - [`InvalidAccountId`](crate::cmd::ApiErrorKind::InvalidAccountId) is returned if the Account Number is syntactically invalid.
/// - [`RateLimitExceeded`](crate::cmd::ApiErrorKind::RateLimitExceeded) can happen for new accounts or when logging into a different account. It means that Obscura is being DoSed and needs to reject new logins.
/// - [`SignupLimitExceeded`](crate::cmd::ApiErrorKind::SignupLimitExceeded) can happen when creating a new account. New accounts are not currently being issued.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AcquireToken {
    pub account_id: AccountId,

    /// Proof-of-work Challenge to Bypass Rate Limits
    pub pow: Option<PowOutput>,
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
      "account_id": "0000000000000000000",
      "pow": {
        "nonce": "f0e1d2c3b4a5",
        "solutions": [
          "000000000000002a",
          "0000000000000457"
        ]
      }
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
