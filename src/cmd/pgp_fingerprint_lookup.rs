use serde::{Deserialize, Serialize};

use super::Cmd;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PgpFingerprintLookup {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PgpFingerprintLookupResponse {
    pub pgp_fingerprints: Vec<String>,
}

impl Cmd for PgpFingerprintLookup {
    type Output = PgpFingerprintLookupResponse;
    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "pgp_fingerprint_lookup";
}

#[test]
fn test_json() {
    crate::cmd::check_cmd_json::<PgpFingerprintLookup>(
        Some(
            r#"
            {
                "email": "me@example.com"
            }
            "#,
        ),
        Some(
            r#"
            {
                "pgp_fingerprints": ["B66B891DD83B0E677D84FC309BB92CC1552E99AA"]
            }
            "#,
        ),
    );
}
