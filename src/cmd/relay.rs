use serde::{Deserialize, Serialize};

use crate::types::OneRelay;

use super::Cmd;

const RELAYS_PATH: &str = "relays";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListRelays {}

impl Cmd for ListRelays {
    type Output = Vec<OneRelay>;
    const METHOD: http::Method = http::Method::GET;
    const PATH: &'static str = RELAYS_PATH;
}

#[test]
fn test_json() {
    let output_json = r#"[
  {
    "id": "NYC-001",
    "ip_v4": "8.8.31.3",
    "ip_v6": "2001:db8:1234:ffff:ffff:ffff:ffff:ffff"
  }
    ]"#;
    crate::cmd::check_cmd_json::<ListRelays>(None, Some(output_json));
}
