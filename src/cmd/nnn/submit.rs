use crate::cmd::{AuthRequired, Cmd};
use crate::types::NnnCaseInfo;
use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::serde_as;

#[serde_as]
#[derive(Clone, Deserialize, Serialize)]
pub struct NnnSubmit {
    pub claimed_months: u16,
    pub email: Option<String>,
    #[serde_as(as = "Base64")]
    pub file: Vec<u8>,
    pub original_file_name: String,
    pub vpn_provider: Option<String>,
}

impl std::fmt::Debug for NnnSubmit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            claimed_months,
            email,
            file,
            original_file_name,
            vpn_provider,
        } = self;
        f.debug_struct("NnnSubmit")
            .field("claimed_months", claimed_months)
            .field("email", &email.as_ref().map(|_| format_args!("_")))
            .field("file_len", &file.len())
            .field("original_file_name", original_file_name)
            .field("vpn_provider", vpn_provider)
            .finish()
    }
}

impl Cmd for NnnSubmit {
    type Output = NnnCaseInfo;
    type Auth = AuthRequired;
    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "nnn/submit";
}

#[test]
fn test_json() {
    crate::cmd::check_cmd_json::<NnnSubmit>(
        Some(
            r#"
            {
                "claimed_months": 12,
                "email": "me@example",
                "file": "aGVsbG8=",
                "original_file_name": "receipt.pdf",
                "vpn_provider": "Obscura"
            }
            "#,
        ),
        Some(
            r#"
            {
                "applied_months": null,
                "claimed_months": 12,
                "share_id": "abcde"
            }
            "#,
        ),
    );

    let cmd: NnnSubmit = serde_json::from_str(
        r#"
        {
            "claimed_months": 1,
            "email": "me@example",
            "file": "aGVsbG8=",
            "original_file_name": "receipt.pdf",
            "vpn_provider": "Obscura"
        }
        "#,
    )
    .unwrap();
    assert_eq!(cmd.file, b"hello");
    assert_eq!(
        format!("{cmd:?}"),
        r#"NnnSubmit { claimed_months: 1, email: Some(_), file_len: 5, original_file_name: "receipt.pdf", vpn_provider: Some("Obscura") }"#
    );
}
