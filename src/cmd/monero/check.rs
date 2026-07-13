use crate::cmd::Cmd;
use serde::{Deserialize, Serialize};

#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckMoneroTopUpInfo {
    #[deprecated]
    pub amount_expected: f64,
    #[deprecated]
    pub amount_received: f64,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub expected_piconero: u64,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub received_piconero: u64,
    pub status: MoneroPaymentStatus,
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
      "amount_expected": 0.02,
      "amount_received": 0.02,
      "expected_piconero": "20000000000",
      "received_piconero": "20000000000",
      "status": "paid"
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
      "amount_expected": 0.002,
      "amount_received": 0.002,
      "expected_piconero": "2000000000",
      "received_piconero": "2000000000",
      "status": "confirming"
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
      "amount_expected": 0.05,
      "amount_received": 0.0,
      "expected_piconero": "50000000000",
      "received_piconero": "0",
      "status": "unpaid"
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
      "amount_expected": 0.05,
      "amount_received": 0.04,
      "expected_piconero": "50000000000",
      "received_piconero": "40000000000",
      "status": "partially_paid"
    }
    "#;
    crate::cmd::check_cmd_json::<CheckMoneroTopUp>(Some(cmd_json), Some(output_json));
}
