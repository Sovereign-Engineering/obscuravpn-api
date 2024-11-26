use serde::{Deserialize, Serialize};

use crate::types::OneExit;

use super::Cmd;

const EXITS_PATH: &str = "exits";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListExits {}

impl Cmd for ListExits {
    type Output = Vec<OneExit>;
    const METHOD: http::Method = http::Method::GET;
    const PATH: &'static str = EXITS_PATH;
}

#[test]
fn test_json() {
    let output_json = r#"
    [
      {
        "id": "NYC-001",
        "country_code": "US",
        "city_name": "New York"
      }
    ]
    "#;
    crate::cmd::check_cmd_json::<ListExits>(None, Some(output_json));
}
