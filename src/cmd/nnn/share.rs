use crate::cmd::Cmd;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NnnShareId(pub String);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NnnShare {
    pub share_id: NnnShareId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NnnShareOutput {
    pub claimed_months: u16,
    pub vpn_provider: Option<String>,
}

impl Cmd for NnnShare {
    type Output = NnnShareOutput;
    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "nnn/share";
}

#[test]
fn test_json() {
    crate::cmd::check_cmd_json::<NnnShare>(
        Some(
            r#"
            {
                "share_id": "abcde"
            }
            "#,
        ),
        Some(
            r#"
            {
                "claimed_months": 12,
                "vpn_provider": "Obscura"
            }
            "#,
        ),
    );
}
