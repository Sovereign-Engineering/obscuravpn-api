use crate::cmd::Cmd;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckMoneroTopUpInfo {
    pub status: MoneroPaymentStatus,
    pub amount_expected: f64,
    pub amount_received: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MoneroPaymentStatus {
    Unpaid,
    PartiallyPaid,
    Confirming,
    Paid,
    Expired,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoneroTopUpId(pub String);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckMoneroTopUp {
    pub id: MoneroTopUpId,
}

impl Cmd for CheckMoneroTopUp {
    type Output = CheckMoneroTopUpInfo;

    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "monero/top_up/check";
}

#[test]
fn test_json_paid() {
    let cmd_json = r#"
    {
        "id": "asdf"
    }
    "#;
    let output_json = r#"
    {
      "status": "paid",
      "amount_expected": 0.02,
      "amount_received": 0.02
    }
    "#;
    crate::cmd::check_cmd_json::<CheckMoneroTopUp>(Some(cmd_json), Some(output_json));
}

#[test]
fn test_json_confirming() {
    let cmd_json = r#"
    {
        "id": "asdf"
    }
    "#;
    let output_json = r#"
    {
      "status": "confirming",
      "amount_expected": 0.002,
      "amount_received": 0.002
    }
    "#;
    crate::cmd::check_cmd_json::<CheckMoneroTopUp>(Some(cmd_json), Some(output_json));
}

#[test]
fn test_json_unpaid() {
    let cmd_json = r#"
    {
        "id": "asdf"
    }
    "#;
    let output_json = r#"
    {
      "status": "unpaid",
      "amount_expected": 0.05,
      "amount_received": 0.0
    }
    "#;
    crate::cmd::check_cmd_json::<CheckMoneroTopUp>(Some(cmd_json), Some(output_json));
}

#[test]
fn test_json_bad_payment() {
    let cmd_json = r#"
    {
        "id": "asdf"
    }
    "#;
    let output_json = r#"
    {
      "status": "partially_paid",
      "amount_expected": 0.05,
      "amount_received": 0.04
    }
    "#;
    crate::cmd::check_cmd_json::<CheckMoneroTopUp>(Some(cmd_json), Some(output_json));
}
