use serde::{Deserialize, Serialize};

use crate::{cmd::ETagCmd, types::OneExit};

use super::Cmd;

const EXITS_PATH: &str = "exits";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListExits {}

impl Cmd for ListExits {
    type Output = Vec<OneExit>;
    const METHOD: http::Method = http::Method::GET;
    const PATH: &'static str = EXITS_PATH;
}

impl ETagCmd for ListExits {}

#[test]
fn test_json() {
    let output_json = r#"
    [
      {
        "id": "NYC-001",
        "country_code": "US",
        "city_code": "nyc",
        "city_name": "New York",
        "provider_id": "us-nyc-wg-301",
        "provider_url": "https://mullvad.net/servers",
        "provider_name": "Mullvad VPN",
        "provider_homepage_url": "https://mullvad.net"
      }
    ]
    "#;
    crate::cmd::check_cmd_json::<ListExits>(None, Some(output_json));
}
