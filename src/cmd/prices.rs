use super::Cmd;
use crate::{cmd::ETagCmd, types::Prices};
use serde::{Deserialize, Serialize};

const PRICES_PATH: &str = "prices";

// WARN: Client doesn't support GET parameters
// https://linear.app/soveng/issue/OBS-2002/support-get-query-parameters-in-rust-api-client
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListPrices {
    pub code: Option<String>,
}

impl Cmd for ListPrices {
    type Output = Prices;
    const METHOD: http::Method = http::Method::GET;
    const PATH: &'static str = PRICES_PATH;
}

impl ETagCmd for ListPrices {}

#[test]
fn test_json() {
    let output_json = r#"
    {
        "subscription": [{
            "months": 1,
            "usd_cents": 800,
            "regular_usd_cents": 800,
            "sale": null
        }, {
            "months": 12,
            "usd_cents": 8000,
            "regular_usd_cents": 9600,
            "sale": {
                "id": "year",
                "title": "Yearly Discount",
                "summary": "Yearly Subscritpion Discount"
            }
        }],
        "top_up": [{
            "months": 1,
            "usd_cents": 800,
            "regular_usd_cents": 800,
            "sale": null
        }],
        "sale": null
    }
    "#;
    crate::cmd::check_cmd_json::<ListPrices>(None, Some(output_json));
}
