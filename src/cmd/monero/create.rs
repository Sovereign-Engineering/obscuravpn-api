use crate::{
    cmd::{Cmd, MoneroTopUpId},
    types::SaleId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoneroTopUpInfo {
    pub id: MoneroTopUpId,

    /// The `monero:` URI describing the payment to be made.
    pub pay_uri: String,
}

/// Create a Monero Top-up.
///
/// This locks in the price conversion. A top-up does expire so should be used immediately (broadcasting the payment within an hour is safe). If funds are paid after a top-up expires the user will have to contact support to try and come to a resolution.
///
/// ## Expected Errors
/// - [`SaleNotFound`](crate::cmd::ApiErrorKind::SaleNotFound)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateMoneroTopUp {
    /// The amount of months to purchase.
    ///
    /// This must be at least 3 and an allowed value from the [`/api/prices` endpoint](crate::cmd::ListPrices).
    ///
    // TODO: https://linear.app/soveng/issue/OBS-3809/indicate-in-api-what-month-amounts-are-allowed-for-monero
    pub months: u16,

    /// Sale ID for this price.
    ///
    /// If specified it *must* apply to this price or the request will be rejected with [`SaleNotFound`](crate::cmd::ApiErrorKind::SaleNotFound).
    pub sale: Option<SaleId>,
}

impl Cmd for CreateMoneroTopUp {
    type Output = MoneroTopUpInfo;

    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = "monero/top_up";
}
