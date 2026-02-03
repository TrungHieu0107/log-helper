//! Encoding utilities for converting SHIFT-JIS to UTF-8.
//!
//! Uses encoding_rs crate instead of Windows APIs for portability.

use encoding_rs::SHIFT_JIS;
use std::io::Read;

/// Convert SHIFT-JIS encoded bytes to UTF-8 string.
pub fn shift_jis_to_utf8(data: &[u8]) -> String {
    let (decoded, _, had_errors) = SHIFT_JIS.decode(data);
    if had_errors {
        // Log warning but continue - some bytes may not be valid SHIFT-JIS
    }
    decoded.into_owned()
}

/// Read a file with SHIFT-JIS encoding and return as UTF-8 string.
pub fn read_file_as_utf8(file_path: &str) -> std::io::Result<String> {
    let mut file = std::fs::File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(shift_jis_to_utf8(&buffer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_passthrough() {
        // Pure ASCII/UTF-8 should pass through unchanged
        let data = b"Hello, World!";
        let result = shift_jis_to_utf8(data);
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_shift_jis_conversion() {
        // SHIFT-JIS encoding of "日本語" (Japanese text)
        let data: &[u8] = &[0x93, 0xFA, 0x96, 0x7B, 0x8C, 0xEA];
        let result = shift_jis_to_utf8(data);
        assert_eq!(result, "日本語");
    }
}
