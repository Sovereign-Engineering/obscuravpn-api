use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Check {
    // The requesting IP is an implicit parameter.
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum IpType {
    Mullvad,

    /// We don't know about this IP address.
    Unknown,

    /// Catch-all, never returned but let's old clients parse new responses.
    ///
    /// This means the server could identify the IP but the client doesn't know about this new type.
    #[serde(other)]
    Other,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CheckResult {
    /// The user is using an exit that may be Obscura traffic.
    ///
    /// Either way the user is not coming from an unknown IP range.
    pub is_safe: bool,

    pub ip: String,
    pub ip_type: IpType,
}

impl Check {
    pub fn to_request(&self, base_url: impl AsRef<str>) -> anyhow::Result<http::Request<()>> {
        let url = Url::parse(base_url.as_ref())?.join("check")?;
        Ok(http::Request::get(url.as_str()).body(())?)
    }
}

#[test]
fn check_err_json() {
    assert_eq!(
        serde_json::to_string(&CheckResult {
            is_safe: true,
            ip: "10.0.1.2".into(),
            ip_type: IpType::Mullvad
        })
        .unwrap(),
        r#"{"is_safe":true,"ip":"10.0.1.2","ip_type":"Mullvad"}"#,
    );

    assert_eq!(
        serde_json::from_str::<CheckResult>(
            r#"
            {
                "is_safe": false,
                "ip": "10.9.8.7",
                "ip_type": "Unknown"
            }
        "#
        )
        .unwrap(),
        CheckResult {
            is_safe: false,
            ip: "10.9.8.7".into(),
            ip_type: IpType::Unknown
        }
    );

    assert_eq!(
        serde_json::from_str::<CheckResult>(
            r#"
            {
                "is_safe": false,
                "ip": "127.0.0.1",
                "ip_type": "NewValue"
            }
        "#
        )
        .unwrap(),
        CheckResult {
            is_safe: false,
            ip: "127.0.0.1".into(),
            ip_type: IpType::Other
        }
    );
}
