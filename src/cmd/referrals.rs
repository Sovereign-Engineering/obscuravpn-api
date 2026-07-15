use serde::{Deserialize, Serialize};

use super::Cmd;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReferralCode(pub String);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UseReferralCode {
    pub code: ReferralCode,
}

impl Cmd for UseReferralCode {
    type Output = ();
    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "referral/use";
}

#[test]
fn test_use_referral_json() {
    crate::cmd::check_cmd_json::<UseReferralCode>(
        Some(
            r#"
            {
                "code": "abc"
            }
            "#,
        ),
        None,
    );
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RotateReferralCode {}

impl Cmd for RotateReferralCode {
    type Output = ();
    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "referral/rotate";
}

#[test]
fn test_rotate_referral_code_json() {
    crate::cmd::check_cmd_json::<RotateReferralCode>(
        Some(
            r#"
            {}
            "#,
        ),
        None,
    );
}
