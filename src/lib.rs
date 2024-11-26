#![allow(clippy::redundant_closure)]

pub mod check;
pub mod cmd;
pub mod token;
pub mod types;
pub mod wg_conf;

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "client")]
pub mod notices;
#[cfg(feature = "client")]
pub use client::Client;
#[cfg(feature = "client")]
pub use client::ClientError;
