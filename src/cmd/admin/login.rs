use crate::types::AuthToken;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdminLoginOutput {
    pub auth_token: AuthToken,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminLogin {
    pub username: String,
    pub password: String,
}

impl AdminLogin {
    pub fn to_request2(&self, base_url: impl AsRef<str>) -> anyhow::Result<http::Request<String>> {
        let url = Url::parse(base_url.as_ref())?.join("admin/login")?;
        Ok(http::Request::post(url.as_str())
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(self)?)?)
    }
}
