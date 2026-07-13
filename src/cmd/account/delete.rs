use crate::cmd::Cmd;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeleteAccountOutput {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeleteAccount {}

impl Cmd for DeleteAccount {
    type Output = DeleteAccountOutput;

    const METHOD: http::Method = http::Method::DELETE;
    const PATH: &'static str = "account";
}
