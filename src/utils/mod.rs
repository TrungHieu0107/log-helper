//! Utility modules for file I/O, encoding, clipboard, and SQL connectivity.

pub mod clipboard;
pub mod encoding;
pub mod file_helper;

#[cfg(feature = "sql")]
pub mod sql_connector;
