use crate::cmd::Cmd;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppleAssociateAccountOutput {}

/// Associate an Apple account with an Obscura account.
///
/// This makes it so that Apple subscriptions fund the associated account.
///
/// Each Apple account can be associated with at most one Obscura account. Associating with a new account will disassociate any previous account.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppleAssociateAccount {
    /// The encoded app transaction.
    ///
    /// Specifically it is the [`jwsRepresentation`](https://developer.apple.com/documentation/storekit/verificationresult/jwsrepresentation-6ma59) field of [`AppTransaction.shared`](https://developer.apple.com/documentation/storekit/apptransaction/shared).
    pub app_transaction_jws: String,
}

impl Cmd for AppleAssociateAccount {
    type Output = AppleAssociateAccountOutput;

    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "apple/associate_account";
}

#[test]
fn test_json() {
    crate::cmd::check_cmd_json::<AppleAssociateAccount>(
        Some(
            r#"
            {
                "app_transaction_jws": "thisisdefinitelyavalidjws"
            }
            "#,
        ),
        Some("{}"),
    );
}
