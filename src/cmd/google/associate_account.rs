use crate::cmd::Cmd;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GoogleAssociateAccount {
    pub purchase_token: String,
    pub promo_code: Option<String>,
}

impl Cmd for GoogleAssociateAccount {
    type Output = ();

    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "google/associate_account";
}
