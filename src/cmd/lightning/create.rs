use crate::{cmd::Cmd, types::SaleId};
use serde::{Deserialize, Serialize};

const LIGHTNING_TOP_UP_PATH: &str = "lightning/top_up";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LightningTopUpInfo {
    /// The payment reference.
    ///
    /// In addition to being used in API responses it can be used when contacting support. The payment reference (or first 4 characters) should be stored and shown to users so that they can provide to support when seeking help with a payment.
    pub id: String,

    /// The serialized Lightning invoice.
    pub invoice: String,

    /// The required payment amount.
    ///
    /// This is derived from the invoice and is just provided for convenience.
    pub millisatoshis: u64,
}

/// Create a Lightning Top-up.
///
/// This locks in the price conversion. A top-up does expire so should be used immediately (within an hour is safe). If funds are paid after a top-up expires the user will have to contact support to try and come to a resolution.
///
/// ## Expected Errors
/// - [`SaleNotFound`](crate::cmd::ApiErrorKind::SaleNotFound)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateLightningTopUp {
    /// The amount of months to purchase.
    ///
    /// This must be an allowed value from the [`/api/prices` endpoint](crate::cmd::ListPrices).
    pub months: u16,

    /// Sale ID for this price.
    ///
    /// If specified it *must* apply to this price or the request will be rejected with [`SaleNotFound`](crate::cmd::ApiErrorKind::SaleNotFound).
    pub sale: Option<SaleId>,
}

impl Cmd for CreateLightningTopUp {
    type Output = LightningTopUpInfo;

    const METHOD: http::Method = http::Method::POST;
    const PATH: &'static str = LIGHTNING_TOP_UP_PATH;
}
