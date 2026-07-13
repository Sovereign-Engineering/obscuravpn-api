use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use std::{fmt, net};

use base64::prelude::*;
use ipnetwork;
use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::serde_as;
use thiserror::Error;

use crate::cmd::{MoneroPaymentStatus, MoneroTopUpId};
use crate::time::Timestamp;

/// Account Number
///
/// Note: In code this is referred to as "Account ID" but for user facing purposes it must always be referred to as "Account Number".
///
/// The Account ID is the primary identifier for an Obscura account. It also serves as the only credential to access the account. This is a relatively weak security system but it is justified as it minimizes the amount of information Obscura needs to store.
///
/// Currently account numbers are 20 digit numbers where the last digit is a Verhoeff checksum. However to allow forwards compatibility and avoid floating-point precision errors the API transacts in the string form. It is encouraged to use the string form as much as possible, only relying on the known format to provide better error messages on user input. In cases where network latency is acceptable the API will always give you up-to-date validation errors on the [`/api/token2` endpoint](crate::token::AcquireToken).
///
/// # Formatting
///
/// There are two primary forms of the Account Number "formatted" and "unformatted".
///
/// ## Formatted
///
/// The formatted Account Number is the only form that should be seen by the user in regular operation. It is the account number separated into 5 groups of 4 characters with ` - ` separation. Example `0123 - 4567 - 8901 - 2345 - 6789`.
///
/// If there are more than 20 characters the remaining characters should be added to the final section without separation like `0123 - 4567 - 8901 - 2345 - 67899999`. This should only happen during user input but may also occur if the Account Number format is changed.
///
/// ## Unformatted
///
/// This form is only used in API requests and responses. It should not be shown to the user.
///
/// You can get to this form from the formatted form by stripping all spaces and dashes.
///
/// ## Placeholder
///
/// If you require a roughly-correct-length placeholder use `XXXX - XXXX - XXXX - XXXX - XXXX`.
///
/// # Usage
///
/// Since this value is sensitive it has a handful of rules.
/// 1. Use reasonable protection when storing this value.
/// 1. Do not share with third parties.
/// 1. This value should almost never be displayed.
///     - The [Account Number Hint](AccountId::hint) may be displayed more freely.
///     - Exception: It can be displayed when first generated.
///     - Exception: It can be displayed after explicit user action (example a "click to reveal" button).
///         - Hovering is not a sufficiently explicit user action.
///     - Exception: It is fine to show a user-entered number as they input it.
#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct AccountId(String);

impl AccountId {
    pub fn from_string_unchecked(id: String) -> Self {
        AccountId(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The digit portion of the Account Number Hint.
    ///
    /// ## Formatting
    ///
    /// The hint is the first three characters of the account number.
    ///
    /// The full hint should be displayed with a trailing ellipsis (`123...`).
    ///
    /// ## Usage
    ///
    /// The hint must not be shared or shown publicly but is suitable for use where it is unlikely for anyone but the user to see it. Including the following scenarios:
    ///
    /// 1. On the screen without prompt. (Ideally hidden if screen capture/sharing is detected.)
    /// 2. In otherwise sensitive documents and information such as payment logs and billing records.
    /// 3. Sent in private communication to the user (such as an email or other direct message).
    pub fn hint(&self) -> &str {
        // We don't currently enforce correctness for the account ID so this method needs to be forgiving.

        match self.0.char_indices().nth(3) {
            Some((i, _)) => &self.0[..i],
            None => &self.0,
        }
    }

    /// Unformatted Account Number.
    pub fn as_string(self) -> String {
        self.0
    }
}

#[test]
fn test_account_id_hint() {
    assert_eq!(AccountId::from_string_unchecked("".into()).hint(), "");
    assert_eq!(AccountId::from_string_unchecked("a".into()).hint(), "a");
    assert_eq!(AccountId::from_string_unchecked("ab".into()).hint(), "ab");
    assert_eq!(AccountId::from_string_unchecked("abc".into()).hint(), "abc");
    assert_eq!(AccountId::from_string_unchecked("abcd".into()).hint(), "abc");
    assert_eq!(AccountId::from_string_unchecked("30493434963831824517".into()).hint(), "304");
    assert_eq!(
        AccountId::from_string_unchecked("3049343496383182451730493434963831824517".into()).hint(),
        "304"
    );

    assert_eq!(AccountId::from_string_unchecked("😛🤠🙃😉".into()).hint(), "😛🤠🙃");
}

impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::fmt::Debug for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AccountId({}...)", self.hint())
    }
}

/// Information about an account.
///
/// ## Funding
///
/// The primary flag is [`active`](AccountInfo::active). This is **the** canonical indicator of whether an account is currently funded. The other information is just supplemental to provide more context to the user.
///
/// In addition [`auto_renews`](AccountInfo::auto_renews) and [`current_expiry`](AccountInfo::current_expiry) provide high-level information about account expiry and future funding.
///
/// Past that the information is all funding-method specific. The [`primary_funding`](AccountInfo::primary_funding) attribute will indicate which payment method is currently (or most recently) funding the account. However when working with this field (as well as other funding-method-specific fields) it is important to remember that new methods may be added at any time so your code has to gracefully handle the case where it doesn't understand the primary or any active funding method. In that case it must fall back to the high-level attributes rather than declaring the account unfunded or similar.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AccountInfo {
    /// High-level account status.
    ///
    /// An active account can use the service. An inactive account can't do much, mostly just use payment-related endpoints to become active.
    ///
    /// This flag is set iff at least one funding method is `active: true`. However, always assume that funding methods you don't know about may be added. Your client must handle this top-level active flag gracefully even if you can't tell how the account is being funded.
    pub active: bool,

    pub apple_subscription: Option<AppleSubscriptionInfo>,

    /// The auto-renew state and time.
    ///
    /// If `Some` the account is auto-renewing and is expected to stay active indefinitely. In this case the timestamp is the time at which the next charge will be made.
    pub auto_renews: Option<Timestamp>,

    /// Unless action is taken the account will expire at this date.
    ///
    /// If auto-renewing or inactive, this is None
    pub current_expiry: Option<Timestamp>,

    /// If the user expects to receive a bonus month when they make a payment.
    ///
    /// This would typically be due to a referral bonus.
    pub free_month_on_next_payment: bool,

    #[serde(default)]
    pub google_subscription: Option<GoogleSubscriptionInfo>,

    /// True if the user made any payment in the past.
    pub has_paid: bool,

    /// If the user has an active referral that will be used for their first purchase.
    ///
    /// This value will be reset to `false` at some point after the first purchase is made.
    pub has_referrer: bool,

    pub id: AccountId,

    /// Monero top-ups that are currently in progress.
    ///
    /// Lists top-ups for which a payment has been detected but that has not yet been fully credited to the account. This lets you discover Monero payments that may be credited to the account soon.
    pub monero_pending_payments: Vec<MoneroPaymentInProgress>,

    /// The most relevant funding method for the account.
    ///
    /// This is not specifically defined and the policy may change over time as payment methods and implementations change. However it is the go-to field for what funding method should be highlighted in any sort of interface that doesn't list them all.
    ///
    /// For example an account with top-up credit and a Stripe Subscription would have their subscription payments paused until their top-up credit runs out, so this would indicate the top-up credit until it expires at which point it would return to the Stripe Subscription. Do not assume the priority order of payments as it can change at any time. This field is the only reliable way to know the priority.
    ///
    /// Invariants:
    /// - If the account is active this will point at an active method.
    /// - If this is `None` the account has never been funded or hasn't been funded recently.
    pub primary_funding: Option<FundingMethod>,

    /// The number of free months the user has been given.
    ///
    /// This counts both bonus months for referees and reward months for referrers as well as any future grant reasons that are added.
    pub received_free_months: u32,

    /// A code the user can use to refer another user.
    ///
    /// For sharing the URL `https://obscura.com/refer#{code}` can be used.
    pub referral_code: String,

    pub stripe_subscription: Option<StripeSubscriptionInfo>,
    #[cfg(feature = "server")]
    pub subscription: Option<StripeSubscriptionInfo>,
    pub top_up: Option<TopUp>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FundingMethod {
    AppleSubscription,
    GoogleSubscription,
    StripeSubscription,

    /// The account has been topped-up with credit.
    ///
    /// This can come from various sources such as one-time payments, vouchers or bonuses.
    TopUp,

    /// The client doesn't understand the funding method. Never emitted.
    #[serde(other)]
    Unknown,
}

#[test]
fn test_funding_method() {
    assert_eq!(serde_json::to_string(&FundingMethod::TopUp).unwrap(), r#""top_up""#,);

    assert_eq!(
        serde_json::from_str::<FundingMethod>(
            r#"
                "top_up"
            "#
        )
        .unwrap(),
        FundingMethod::TopUp,
    );

    assert_eq!(
        serde_json::from_str::<FundingMethod>(
            r#"
                "snail_mail"
            "#
        )
        .unwrap(),
        FundingMethod::Unknown,
    );
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TopUp {
    /// If the account is active because of a top-up.
    pub active: bool,

    /// Top-up expiry time.
    pub credit_expires_at: Timestamp,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StripeSubscriptionInfo {
    pub active: bool,
    /// string repr of [`stripe::SubscriptionStatus`](https://docs.rs/async-stripe/latest/stripe/enum.SubscriptionStatus.html)
    pub status: String,
    /// period start in seconds since unix epoch
    pub current_period_start: Timestamp,
    /// period end in seconds since unix epoch
    pub current_period_end: Timestamp,
    /// whether the subscription will end at this period
    pub cancel_at_period_end: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SubscriptionTarget {
    Email { addr: String },
    Nostr { addr: String },
}

impl SubscriptionTarget {
    pub fn email(&self) -> Option<&str> {
        let Self::Email { addr } = self else { return None };
        Some(addr)
    }
    pub fn nostr(&self) -> Option<&str> {
        let Self::Nostr { addr } = self else { return None };
        Some(addr)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppleSubscriptionInfo {
    /// If `true` this subscription is currently funding the account.
    pub active: bool,

    /// Whether the subscription will renew automatically
    pub auto_renew_status: bool,

    /// Date of expiration or next renewal.
    pub renewal_date: Timestamp,

    /// Raw subscription status from Apple.
    ///
    /// <https://developer.apple.com/documentation/appstoreserverapi/status>
    ///
    /// This can be used to show more information about the subscription but wherever possible the provided fields (such as [active](AppleSubscriptionInfo::active)) should be used instead to ensure that your UI matches the server's opinion.
    pub status: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GoogleSubscriptionInfo {
    /// Whether the subscription is active
    pub active: bool,

    /// Whether the subscription will renew automatically
    pub auto_renew_status: bool,

    /// Date of expiration or next renewal.
    ///
    /// `None` for `PENDING` subscriptions where Google has not yet
    /// granted access (and therefore not assigned an expiry).
    pub expires_at: Option<Timestamp>,

    /// Serialized SubscriptionState
    ///
    /// <https://developers.google.com/android-publisher/api-ref/rest/v3/purchases.subscriptionsv2#SubscriptionState>
    ///
    /// Example: `SUBSCRIPTION_STATE_ACTIVE` or `SUBSCRIPTION_STATE_CANCELED`.
    ///
    /// This can be used to show more information about the subscription but wherever possible the provided fields (such as [active](GoogleSubscriptionInfo::active)) should be used instead to ensure that your UI matches the server's opinion.
    pub status: String,
}

/// The maximum length that a tunnel label can be set to.
///
/// Clients should not assume anything about labels that they receive, except that they can comfortably fit in memory on basically any modern device.
pub const TUNNEL_LABEL_MAX_BYTES: usize = 128;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OneTunnel {
    pub id: String,

    /// A user-provided label for the tunnel.
    ///
    /// If None, the user did not provide a label and the client should show some nice representation of this, for example the ID and last active time.
    ///
    /// The string itself will never be empty (0 bytes) but clients may wish consider cases such as all whitespace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub status: TunnelStatus,
    pub config: TunnelConfig,
    pub relay: OneRelay,
    pub exit: OneExit,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TunnelStatus {
    /// The tunnel has been created but not used yet.
    Created {
        /// Timestamp when this status was last updated.
        when: Timestamp,
    },
    Connected {
        /// Timestamp when this status was last updated.
        ///
        /// Note: Not the timestamp when the tunnel was last connected to.
        when: Timestamp,
    },
    Disconnected {
        /// Timestamp when this status was last updated.
        ///
        /// Note: This typically is the time when the tunnel was disconnected but that shouldn't be assumed.
        when: Timestamp,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TunnelConfig {
    UdpPort { client: WgClientConfig, server: WgServerConfig },
    Obfuscated(ObfuscatedTunnelConfig),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WgClientConfig {
    pub wg_pubkey: WgPubkey,
    pub addresses: Vec<ipnetwork::IpNetwork>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WgServerConfig {
    pub wg_pubkey: WgPubkey,
    pub endpoints: Vec<net::SocketAddr>,
    pub dnses: Vec<net::IpAddr>,
}

fn default_gateway_ip_v4() -> net::Ipv4Addr {
    net::Ipv4Addr::new(10, 64, 0, 1)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ObfuscatedTunnelConfig {
    pub client_pubkey: WgPubkey,
    pub client_ips_v4: Vec<ipnetwork::Ipv4Network>,
    pub client_ips_v6: Vec<ipnetwork::Ipv6Network>,
    pub dns: Vec<net::IpAddr>,
    // TODO: remove after roll out in all environments
    #[serde(default = "default_gateway_ip_v4")]
    pub gateway_ip_v4: net::Ipv4Addr,
    // TODO: Remove. https://linear.app/soveng/issue/OBS-1267
    #[cfg(feature = "server")]
    pub relay_addr_v4: net::SocketAddrV4,
    // TODO: Remove. https://linear.app/soveng/issue/OBS-1267
    #[cfg(feature = "server")]
    pub relay_addr_v6: net::SocketAddrV6,
    // TODO: Remove. https://linear.app/soveng/issue/OBS-1267
    #[cfg(feature = "server")]
    pub relay_cert: String,
    pub exit_pubkey: WgPubkey,
}

const WG_PUBKEY_LENGTH: usize = 32;

#[serde_as]
#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct WgPubkey(#[serde_as(as = "Base64")] pub [u8; WG_PUBKEY_LENGTH]);

impl std::fmt::Debug for WgPubkey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        f.debug_tuple("WgPubKey").field(&STANDARD.encode(self.0)).finish()
    }
}

#[derive(Error, Debug)]
pub enum ParseWgPubkeyError {
    #[error("expected {} bytes, found {}", WG_PUBKEY_LENGTH, .0)]
    InvalidLength(usize),
    #[error("base64 decode err: {:?}", .0)]
    NotBase64(#[from] base64::DecodeError),
}

impl FromStr for WgPubkey {
    type Err = ParseWgPubkeyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        let decoded = STANDARD.decode(s)?;
        let bytes = decoded.try_into().map_err(|d: Vec<u8>| ParseWgPubkeyError::InvalidLength(d.len()))?;
        Ok(WgPubkey(bytes))
    }
}

impl Display for WgPubkey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use base64::{display::Base64Display, engine::general_purpose::STANDARD};
        Base64Display::new(&self.0, &STANDARD).fmt(f)
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, derive_more::Debug, PartialEq, Eq)]
pub struct OneRelay {
    pub id: String,

    #[serde(flatten)]
    pub city_code: CityCode,
    pub city_name: String,

    pub preferred_exits: Vec<RelayPreferredExit>,

    /// Unused. Set for compatibility with old clients.
    ///
    /// https://linear.app/soveng/issue/OBS-1318
    #[cfg(feature = "server")]
    #[serde(default = "localhost_ip_v6")]
    pub ip_v6: net::Ipv6Addr,

    /// The IPv4 address where the QUIC API is available.
    ///
    /// The API is available on all of the ports listed in `ports`.
    pub ip_v4: net::Ipv4Addr,
    pub ports: Vec<u16>,
    /// The TLS cert for the QUIC API.
    #[debug("{:?}", BASE64_STANDARD.encode(tls_cert))]
    #[serde_as(as = "Base64")]
    pub tls_cert: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RelayPreferredExit {
    pub id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OneExit {
    pub id: String,
    #[serde(flatten)]
    pub city_code: CityCode,
    pub city_name: String,
    /// The provider's ID for this server.
    pub provider_id: String,
    /// A URL for this server on the provider's site.
    pub provider_url: String,
    /// The name of the provider for this sever.
    pub provider_name: String,
    /// The homepage of the provider of this server.
    pub provider_homepage_url: String,
    /// ID of the datacenter server is located at.
    #[serde(default = "rand::random")]
    pub datacenter_id: u32,
    /// Higher tier servers should be preferred.
    #[serde(default = "u8::max_value")]
    pub tier: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct CountryCode(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct CityCode {
    pub country_code: CountryCode,
    pub city_code: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Prices {
    pub subscription: Vec<Price>,
    pub top_up: Vec<Price>,

    /// A global sale description.
    ///
    /// The individual prices may also have specific reasons (which may be the same).
    pub sale: Option<Sale>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Price {
    /// The amount of months that that will be purchased.
    pub months: u16,

    /// The actual price of the purchase.
    pub usd_cents: u32,

    /// The "regular" price which discounts can be contrasted against.
    ///
    /// This field is purely informational.
    pub regular_usd_cents: u32,

    /// The sale that is setting these prices.
    ///
    /// If this is present it **must** be passed to the purchase endpoint. Otherwise the charged price may be different than the price displayed.
    pub sale: Option<Sale>,
}

/// Opaque ID not to be displayed to users.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SaleId(pub String);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sale {
    pub id: SaleId,

    /// Example: "Launch Sale"
    pub title: String,

    /// Example: "Thanks for being an early Obscura user. Sign up now and get a special price."
    pub summary: String,
}

#[test]
fn serde_wg_enc_dec() {
    let pk = WgPubkey([
        197, 50, 1, 3, 154, 219, 161, 75, 231, 31, 136, 109, 161, 216, 219, 233, 238, 189, 237, 8, 203, 17, 27, 117, 52, 0, 120, 153, 154, 169, 240,
        56,
    ]);

    let json = "\"xTIBA5rboUvnH4htodjb6e697QjLERt1NAB4mZqp8Dg=\"";
    let pk_from_json: WgPubkey = serde_json::from_str(json).unwrap();
    assert_eq!(pk_from_json, pk);
    let json_from_pk = serde_json::to_string(&pk).unwrap();
    assert_eq!(json_from_pk, json);

    let base64 = &json[1..json.len() - 1];
    let pk_from_base64: WgPubkey = base64.parse().unwrap();
    assert_eq!(pk_from_base64, pk);
    let base64_from_pk = pk.to_string();
    assert_eq!(base64_from_pk, base64);
}

#[derive(Clone, PartialEq, Eq)]
pub struct AuthToken(String);

impl AuthToken {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Debug for AuthToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AuthToken").field(&format_args!("_")).finish()
    }
}

impl From<String> for AuthToken {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<AuthToken> for String {
    fn from(value: AuthToken) -> Self {
        value.0
    }
}

fn localhost_ip_v6() -> net::Ipv6Addr {
    net::Ipv6Addr::LOCALHOST
}

/// A Monero top-up with an in-progress payment.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct MoneroPaymentInProgress {
    pub id: MoneroTopUpId,
    pub status: MoneroPaymentStatus,
}
