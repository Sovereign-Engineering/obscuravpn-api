use crate::cmd::ReferralCode;
use crate::types::{AccountId, AuthToken};
use crate::{cmd::Cmd, pow::PowOutput};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateAccountInfo {
    pub id: AccountId,
    pub token: AuthToken,
}

/// Create an Account
///
/// ## Expected Errors
/// - [`InvalidReferralCode`](crate::cmd::ApiErrorKind::InvalidReferralCode)
/// - [`SignupLimitExceeded`](crate::cmd::ApiErrorKind::SignupLimitExceeded)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateAccount {
    pub pow: Option<PowOutput>,
    pub referral_code: Option<ReferralCode>,
}

impl Cmd for CreateAccount {
    type Output = CreateAccountInfo;
    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "account";
}

#[test]
fn test_account_info_json() {
    crate::cmd::check_cmd_json::<CreateAccount>(
        Some(
            r###"
                {
                    "pow": null,
                    "referral_code": "1234"
                }
            "###,
        ),
        Some(
            r###"
                {
                  "id": "0000000000000000000",
                  "token": "abc-123-xyz"
                }
            "###,
        ),
    );
}
