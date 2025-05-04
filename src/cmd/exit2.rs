use serde::{Deserialize, Serialize};

use crate::{cmd::ETagCmd, types::OneExit};

use super::Cmd;

const EXITS_PATH: &str = "exits2";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListExits2 {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ExitList {
    pub exits: Vec<OneExit>,
}

impl Cmd for ListExits2 {
    type Output = ExitList;
    const METHOD: http::Method = http::Method::GET;
    const PATH: &'static str = EXITS_PATH;
}

impl ETagCmd for ListExits2 {}

#[test]
fn test_json() {
    let output_json = r#"
    {
      "exits": [{
        "id": "NYC-001",
        "country_code": "US",
        "city_code": "nyc",
        "city_name": "New York",
        "datacenter_id": 42,
        "provider_id": "us-nyc-wg-301",
        "provider_url": "https://mullvad.net/servers",
        "provider_name": "Mullvad VPN",
        "provider_homepage_url": "https://mullvad.net",
        "tier": 0
      }]
    }
    "#;
    crate::cmd::check_cmd_json::<ListExits2>(None, Some(output_json));
}
