use serde::{Deserialize, Serialize};

use crate::types::OneExit;

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

#[test]
fn test_json() {
    let output_json = r#"
    {
      "exits": [{
        "id": "NYC-001",
        "country_code": "US",
        "city_code": "nyc",
        "city_name": "New York"
      }]
    }
    "#;
    crate::cmd::check_cmd_json::<ListExits2>(None, Some(output_json));
}
