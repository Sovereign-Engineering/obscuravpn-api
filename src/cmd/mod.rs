mod account;
mod apple;
mod cache_wg_key;
mod exit;
mod exit2;
mod friend_codes;
mod google;
mod lightning;
mod monero;
mod newsletter_subscribe;
mod prices;
mod referrals;
mod relay;
mod stripe;
mod tunnel;

pub use account::*;
pub use apple::*;
pub use cache_wg_key::*;
pub use exit::*;
pub use exit2::*;
pub use friend_codes::*;
pub use google::*;
use http::HeaderValue;
pub use lightning::*;
pub use monero::*;
pub use newsletter_subscribe::*;
pub use prices::*;
pub use referrals::*;
pub use relay::*;
pub use stripe::*;
pub use tunnel::*;

use std::any::Any;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;
use tokio_stream::StreamExt;
use url::Url;

use crate::ClientError;
use crate::pow::PowParams;
use crate::response::Response;
use crate::types::AuthToken;

pub trait Cmd: Serialize + DeserializeOwned + std::fmt::Debug {
    type Output: Serialize + DeserializeOwned + 'static + std::fmt::Debug;
    const METHOD: http::Method;
    const PATH: &'static str;

    fn to_request(&self, base_url: impl AsRef<str>, auth_token: &AuthToken, etag: Option<HeaderValue>) -> anyhow::Result<http::Request<String>> {
        let url = Url::parse(base_url.as_ref())?.join(Self::PATH)?;
        let mut request = http::Request::builder()
            .method(Self::METHOD)
            .uri(url.as_str())
            .header(http::header::AUTHORIZATION, format!("Bearer {}", auth_token.as_str()))
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(String::new())?;
        if let Some(etag) = etag {
            request.headers_mut().insert(http::header::IF_NONE_MATCH, etag);
        }
        if Self::METHOD == http::Method::GET {
            // TODO: Support GET parameters: https://linear.app/soveng/issue/OBS-2002/support-get-query-parameters-in-rust-api-client
        } else {
            *request.body_mut() = serde_json::to_string(self)?;
        }
        Ok(request)
    }
}

pub trait ETagCmd: Cmd {}

#[derive(Clone, Debug, Error)]
#[error("{}", self.body.msg)]
pub struct ApiError {
    pub status: http::StatusCode,
    pub body: ApiErrorBody,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ApiErrorBody {
    /// Typed error codes.
    ///
    /// This is the machine-readable component of the error. All other fields are human-readable and unstable.
    ///
    /// Note that the serialization is `{"ErrorCode": detail}`. For example:
    ///
    /// ```json
    /// {
    ///     "RateLimitExceeded": {
    ///         "pow_challenge": {
    ///             "nonce": "abc",
    ///             "threshold": "000...",
    ///             "puzzles": 10,
    ///         }
    ///     }
    /// }
    /// ```
    pub error: ApiErrorKind,

    /// User-readable message.
    ///
    /// This is a message that is suitable for showing to the end-user if the application does not have specific error handling for this error. It should be used rather than a generic "Unexpected error".
    ///
    /// Note: This is currently always English. In the future the `Accept-Language` header may be used to provide a localized error.
    pub msg: String,

    /// Debugging information, not intended for end-users.
    ///
    /// It is a good idea to log this, especially if the error is unexpected or otherwise unhandled. It isn't expected to be helpful to the end-user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ApiErrorKind {
    /// Account not active.
    ///
    /// The account is not active (funded) but tried to do an action that requires a funded account.
    AccountExpired {},

    /// This account already has a referrer.
    AlreadyReferred {},

    /// Every friend code has already been claimed.
    AllFriendCodesClaimed {},

    AssociateAccountConflict {},

    /// The request doesn't satisfy the preconditions.
    BadRequest {},

    /// This account is not eligible to have a referrer linked.
    IneligibleForReferral {},

    /// An internal error occurred.
    ///
    /// You may retry with exponential backoff and jitter.
    InternalError {},

    /// The provided account ID is syntactically incorrect.
    InvalidAccountId {},

    /// The referral code doesn't exist.
    ///
    /// As a special case this also occurs for a self-referral attempt.
    InvalidReferralCode {},

    LightningTopUpNotFound {},
    MiscUnauthorized {},

    /// The request requires authentication which was not satisfactory.
    ///
    /// You must [authenticate](crate::doc::auth) then retry with authentication. If this request was authenticated the credential is expired or incorrect and you must log in again.
    MissingOrInvalidAuthToken {},

    MoneroTopUpNotFound {},

    /// Attempt to call an API route which doesn't exist.
    ///
    /// Notably this is different than calling a valid route for a resource that doesn't exist. You typoed the URL.
    NoApiRoute {},

    /// Your client is no longer supported.
    ///
    /// Either you are using parameters that are no longer supported or the response format has changed in an incompatible way. You need to update your client to the new API.
    NoLongerSupported {},

    /// No exit matched the requested filters.
    NoMatchingExit {},

    /// Rate limit exceeded.
    ///
    /// The request was not fulfilled as it was attempting to do some action too often.
    ///
    /// For endpoints that require authentication this is almost certainly a per-account limit. For requests that don't require authentication this is probably a global limit.
    ///
    /// You may retry with exponential backoff and jitter.
    RateLimitExceeded {
        /// Proof-of-work challenge.
        ///
        /// If set this request may be retried with the solution to this challenge. Currently PoW is only available on authentication token requests.
        pow_challenge: Option<PowParams>,
    },

    /// A purchase was attempted to be made with a specific offer that is not valid.
    ///
    /// If you are very unlucky it just expired, but this should never happen due to grace periods. Refresh the price list and try again.
    SaleNotFound {},

    /// Obscura is not currently accepting new accounts.
    ///
    /// You may retry with exponential backoff and jitter.
    SignupLimitExceeded {
        /// Proof-of-work challenge.
        ///
        /// This may be used to bypass the signup limit.
        pow_challenge: Option<PowParams>,
    },

    /// Your account has created more tunnels than allowed.
    ///
    /// Delete another tunnel and retry.
    TunnelLimitExceeded {},

    /// Your WireGuard key can't be used.
    ///
    /// Generate a new one and retry.
    WgKeyRotationRequired {},

    /// Unknown error kind.
    ///
    /// This is never emitted. Used for forwards-compatibility on the client.
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

pub async fn parse_response<T: 'static + DeserializeOwned>(res: reqwest::Response) -> Result<Response<T>, ClientError> {
    let status = res.status();
    let etag = res.headers().get(http::header::ETAG).cloned();
    if status == http::StatusCode::NOT_MODIFIED {
        return Ok(Response::new(None, etag));
    }
    let is_json = res
        .headers()
        .get(http::header::CONTENT_TYPE)
        .is_some_and(|h| h.as_bytes() == b"application/json");
    let empty: Box<dyn Any + Send + Sync> = Box::new(());

    const MAX_RESPONSE_SIZE: usize = 5_000_000;
    let body = {
        let mut body = Vec::new();
        let mut chunks = res.bytes_stream();
        loop {
            match chunks.next().await {
                Some(Ok(chunk)) => {
                    if chunk.len() + body.len() > MAX_RESPONSE_SIZE {
                        return Err(ClientError::ResponseTooLarge);
                    }
                    body.extend_from_slice(&chunk);
                }
                None => break body,
                Some(Err(error)) => {
                    return Err(ProtocolError {
                        status,
                        raw: String::new(),
                        source: error.into(),
                    }
                    .into());
                }
            }
        }
    };

    if !is_json {
        Err(ProtocolError {
            status,
            raw: String::from_utf8(body).unwrap_or_default(),
            source: anyhow::anyhow!("Non-JSON {status} response"),
        }
        .into())
    } else if !status.is_success() {
        match serde_json::from_slice(&body) {
            Ok(api_error_body) => Err(ApiError {
                status,
                body: api_error_body,
            }
            .into()),
            Err(error) => Err(ProtocolError {
                status,
                raw: String::from_utf8(body).unwrap_or_default(),
                source: error.into(),
            }
            .into()),
        }
    } else if let Ok(empty) = empty.downcast::<T>() {
        Ok(Response::new(Some(*empty), etag))
    } else {
        match serde_json::from_slice(&body) {
            Ok(success_body) => Ok(Response::new(Some(success_body), etag)),
            Err(error) => Err(ProtocolError {
                status,
                raw: String::from_utf8(body).unwrap_or_default(),
                source: error.into(),
            }
            .into()),
        }
    }
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

#[test]
fn rate_limit_pow_challenge_json() {
    use crate::pow::PowDigest;
    let body = ApiErrorBody {
        error: ApiErrorKind::RateLimitExceeded {
            pow_challenge: Some(PowParams {
                nonce: "abc123".into(),
                threshold: PowDigest([0xab; 32]),
                puzzles: 3,
            }),
        },
        msg: "Rate limit exceeded".into(),
        detail: None,
    };

    assert_eq!(
        serde_json::to_string(&body).unwrap(),
        r#"{"error":{"RateLimitExceeded":{"pow_challenge":{"nonce":"abc123","threshold":"abababababababababababababababababababababababababababababababab","puzzles":3}}},"msg":"Rate limit exceeded"}"#,
    );

    assert_eq!(
        serde_json::from_str::<ApiErrorBody>(
            r#"
            {
                "error": {
                    "RateLimitExceeded": {
                        "pow_challenge": {
                            "nonce": "abc123",
                            "threshold": "abababababababababababababababababababababababababababababababab",
                            "puzzles": 3
                        }
                    }
                },
                "msg": "Rate limit exceeded"
            }
        "#
        )
        .unwrap(),
        body
    );
}
