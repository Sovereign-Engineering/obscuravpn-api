#![allow(clippy::redundant_closure)]
extern crate core;

pub mod check;
pub mod cmd;
pub mod token;
pub mod types;
pub mod wg_conf;

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub mod notices;
pub mod pow;
pub mod relay_protocol;
#[cfg(feature = "client")]
mod response;
pub mod wg_fragment;

#[cfg(feature = "client")]
pub use client::Client;
#[cfg(feature = "client")]
pub use client::ClientError;
#[cfg(feature = "client")]
pub use response::Response;
#[cfg(feature = "client")]
pub mod reexports {
    pub use reqwest;
    pub use rustls;
}
