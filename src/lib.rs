pub mod api_types;
pub mod config;
pub mod error;

#[cfg(feature = "ssr")]
pub mod dnsmasq;
#[cfg(feature = "ssr")]
pub mod server;
#[cfg(feature = "ssr")]
pub mod storage;

#[cfg(feature = "csr")]
pub mod app;
#[cfg(feature = "csr")]
pub mod i18n;
#[cfg(any(feature = "csr", feature = "ssr"))]
pub mod server_fns;
#[cfg(feature = "csr")]
pub mod ui;
