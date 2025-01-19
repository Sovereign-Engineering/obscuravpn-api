use crate::cmd::Cmd;
use crate::types::WgPubkey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheWgKey {
    pub public_key: WgPubkey,
}

impl Cmd for CacheWgKey {
    type Output = ();
    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "cache_wg_key";
}
