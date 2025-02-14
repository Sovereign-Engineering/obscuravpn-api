use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use std::{fmt, net};

use ipnetwork;
use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::serde_as;
use thiserror::Error;

#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct AccountId(String);

impl AccountId {
    pub fn from_string_unchecked(id: String) -> Self {
        AccountId(id)
    }

    // The digit portion of the account hint.
    pub fn hint(&self) -> &str {
        // We don't currently enforce correctness for the account ID so this method needs to be forgiving.

        match self.0.char_indices().nth(3) {
            Some((i, _)) => &self.0[..i],
            None => &self.0,
        }
    }

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub id: AccountId,
    pub active: bool,
    pub top_up: Option<TopUp>,
    pub subscription: Option<Subscription>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopUp {
    pub credit_expires_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Subscription {
    /// string repr of [`stripe::SubscriptionStatus`][https://docs.rs/async-stripe/latest/stripe/enum.SubscriptionStatus.html]
    pub status: String,
    /// period start in seconds since unix epoch
    pub current_period_start: i64,
    /// period end in seconds since unix epoch
    pub current_period_end: i64,
    /// whether the subscription will end at this period
    pub cancel_at_period_end: bool,
}

impl Subscription {
    pub fn new(status: String, current_period_start: i64, current_period_end: i64, cancel_at_period_end: bool) -> Self {
        Self {
            status,
            current_period_start,
            current_period_end,
            cancel_at_period_end,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OneTunnel {
    pub id: String,
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
        when: i64,
    },
    Connected {
        /// Timestamp when this status was last updated.
        ///
        /// Note: Not the timestamp when the tunnel was last connected to.
        when: i64,
    },
    Disconnected {
        /// Timestamp when this status was last updated.
        ///
        /// Note: This typically is the time when the tunnel was disconnected but that shouldn't be assumed.
        when: i64,
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
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct WgPubkey(#[serde_as(as = "Base64")] pub [u8; WG_PUBKEY_LENGTH]);

impl std::fmt::Debug for WgPubkey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
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
        use base64::{engine::general_purpose::STANDARD, Engine as _};
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OneRelay {
    pub id: String,
    pub ip_v4: net::Ipv4Addr,
    pub ip_v6: net::Ipv6Addr,
    pub preferred_exits: Vec<RelayPreferredExit>,
    #[serde_as(as = "Base64")]
    pub tls_cert: Vec<u8>,
    pub ports: Vec<u16>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RelayPreferredExit {
    pub id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OneExit {
    pub id: String,
    pub country_code: String,
    pub city_code: String,
    pub city_name: String,
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
    pub months: u16,
    pub usd_cents: u32,

    /// The "regular" price which discounts can be contrasted against.
    /// This field is purely informational.
    pub regular_usd_cents: u32,

    pub sale: Option<Sale>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sale {
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
