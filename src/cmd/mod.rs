mod account;
mod exit;
mod lightning;
mod prices;
mod relay;
mod stripe;
mod tunnel;

pub use account::*;
pub use exit::*;
pub use lightning::*;
pub use prices::*;
pub use relay::*;
use std::any::Any;
pub use stripe::*;
pub use tunnel::*;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::types::AuthToken;
use crate::ClientError;

pub trait Cmd: Serialize + DeserializeOwned + std::fmt::Debug {
    type Output: Serialize + DeserializeOwned + 'static + std::fmt::Debug;
    const METHOD: http::Method;
    const PATH: &'static str;

    fn to_request(&self, base_url: impl AsRef<str>, auth_token: &AuthToken) -> anyhow::Result<http::Request<String>> {
        let url = Url::parse(base_url.as_ref())?.join(Self::PATH)?;
        let mut request = http::Request::builder()
            .method(Self::METHOD)
            .uri(url.as_str())
            .header(http::header::AUTHORIZATION, format!("Bearer {}", auth_token.as_str()))
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(String::new())?;
        if Self::METHOD != http::Method::GET {
            *request.body_mut() = serde_json::to_string(self)?;
        }
        Ok(request)
    }
}

#[derive(Clone, Debug, Error)]
#[error("{}", self.body.msg)]
pub struct ApiError {
    pub status: http::StatusCode,
    pub body: ApiErrorBody,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ApiErrorBody {
    pub error: ApiErrorKind,
    pub msg: String,

    /// Debugging information, not intended for end-users.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ApiErrorKind {
    AccountExpired {},
    BadRequest {},
    InternalError {},
    MissingOrInvalidAuthToken {},
    NoApiRoute {},
    NoMatchingExit {},
    RateLimitExceeded {},
    SignupLimitExceeded {},
    TunnelLimitExceeded {},

    #[serde(untagged)]
    Unknown(serde_json::Value),
}

#[derive(Error, Debug)]
#[error("Unexpected API response: {source}")]
pub struct ProtocolError {
    pub status: http::StatusCode,
    pub raw: String,
    pub source: anyhow::Error,
}

pub async fn parse_response<T: 'static + DeserializeOwned>(res: reqwest::Response) -> Result<T, ClientError> {
    let is_json = res
        .headers()
        .get(http::header::CONTENT_TYPE)
        .is_some_and(|h| h.as_bytes() == b"application/json");
    if !is_json {
        let status = res.status();
        return match res.text().await {
            Ok(raw) => Err(ClientError::ProtocolError(ProtocolError {
                status,
                raw,
                source: anyhow::anyhow!("Non-JSON {status} response"),
            })),
            Err(err) => Err(ClientError::ProtocolError(ProtocolError {
                status,
                raw: String::new(),
                source: err.into(),
            })),
        };
    }

    let status = res.status();
    if !status.is_success() {
        return Err(ClientError::ApiError(ApiError {
            status,
            body: res.json().await.map_err(|err| {
                ClientError::ProtocolError(ProtocolError {
                    status,
                    raw: String::new(),
                    source: err.into(),
                })
            })?,
        }));
    }
    let empty: Box<dyn Any> = Box::new(());
    if let Ok(empty) = empty.downcast::<T>() {
        return Ok(*empty);
    }
    Ok(res.json().await.map_err(anyhow::Error::new)?)
}

#[cfg(test)]
pub(crate) fn check_cmd_json<T: Cmd>(cmd_json: Option<&str>, output_json: Option<&str>)
where
    <T as Cmd>::Output: 'static,
{
    if T::METHOD == http::Method::GET {
        assert!(cmd_json.is_none())
    } else {
        let cmd_json = cmd_json.unwrap();
        let cmd: T = serde_json::from_str(cmd_json).unwrap();
        let cmd_json: serde_json::Value = serde_json::from_str(cmd_json).unwrap();
        assert_eq!(cmd_json, serde_json::to_value(cmd).unwrap());
    }
    let empty: &dyn Any = &();
    if empty.is::<T::Output>() {
        assert!(output_json.is_none())
    } else {
        let output_json = output_json.unwrap();
        let output: T::Output = serde_json::from_str(output_json).unwrap();
        let output_json: serde_json::Value = serde_json::from_str(output_json).unwrap();
        assert_eq!(output_json, serde_json::to_value(output).unwrap());
    }
}

#[test]
fn check_err_json() {
    assert_eq!(
        serde_json::to_string(&ApiErrorBody {
            error: ApiErrorKind::AccountExpired {},
            msg: "Account Expired".into(),
            detail: None,
        })
        .unwrap(),
        r#"{"error":{"AccountExpired":{}},"msg":"Account Expired"}"#,
    );

    assert_eq!(
        serde_json::from_str::<ApiErrorBody>(
            r#"
            {
                "error": {"AccountExpired": {"future_field": 7}},
                "msg": "Account Expired",
                "future field": true
            }
        "#
        )
        .unwrap(),
        ApiErrorBody {
            error: ApiErrorKind::AccountExpired {},
            msg: "Account Expired".into(),
            detail: None,
        }
    );

    assert_eq!(
        serde_json::to_string(&ApiErrorBody {
            error: ApiErrorKind::Unknown(serde_json::Value::String("Other".into())),
            msg: "Foo".into(),
            detail: Some("Extra help".into()),
        })
        .unwrap(),
        r#"{"error":"Other","msg":"Foo","detail":"Extra help"}"#,
    );

    assert_eq!(
        serde_json::from_str::<ApiErrorBody>(
            r#"
            {
                "error": {"FutureVariant": {"future_field": 7}},
                "msg": "Helpful message",
                "future field": true,
                "detail": "debug"
            }
        "#
        )
        .unwrap(),
        ApiErrorBody {
            error: ApiErrorKind::Unknown(
                serde_json::from_str(
                    r#"
                {"FutureVariant": {"future_field": 7}}
            "#
                )
                .unwrap()
            ),
            msg: "Helpful message".into(),
            detail: Some("debug".to_string()),
        }
    );

    assert_eq!(
        serde_json::from_str::<ApiErrorBody>(
            r#"
            {
                "error": "Other",
                "msg": "Helpful message",
                "future field": true
            }
        "#
        )
        .unwrap(),
        ApiErrorBody {
            error: ApiErrorKind::Unknown(serde_json::Value::String("Other".into())),
            msg: "Helpful message".into(),
            detail: None,
        }
    );
}
