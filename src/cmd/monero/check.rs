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
    /// A sufficient payment is detected but is awaiting confirmation.
    Confirming,

    /// This top-up has expired.
    ///
    /// This top-up is expired and must not be used. Create a new top-up to fund your account.
    ///
    /// This is a terminal state.
    Expired,

    /// The top-up has failed.
    ///
    /// This is a terminal state. Contact support to see if we can help you.
    Failed,

    /// The top-up has succeeded.
    ///
    /// The account has been credited.
    ///
    /// This is a terminal state.
    Paid,

    /// A payment is detected but it isn't sufficient.
    ///
    /// This is a terminal state, additional payments will not be processed. Contact support to see if we can help you.
    PartiallyPaid,

    /// Base state, no activity detected.
    Unpaid,
}

/// A Monero top-up ID
///
/// This is used both in the API and can be used when contacting support. This is the main payment reference and should be retained in case any issues come up.
///
/// The first 4 characters can be used when contacting support, the full ID must be used with the API.
///
/// The status of the top-up can be checked with [`CheckMoneroTopUp`].
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoneroTopUpId(pub String);

/// Check the status of a Monero Top-up
///
/// Monero transactions require blockchain confirmations before being processed and typically complete within 40min.
///
/// Once the top-up [`status`](CheckMoneroTopUpInfo::status) is [`Paid`](MoneroPaymentStatus::Paid) it has been processed the account has already been credited.
///
/// Top-up records are eventually purged for privacy reasons. Expect this API to fail for old top-ups.
///
/// ## Expected Errors
/// - [`MoneroTopUpNotFound`](crate::cmd::ApiErrorKind::MoneroTopUpNotFound) if the `id` is invalid or pruned.
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
