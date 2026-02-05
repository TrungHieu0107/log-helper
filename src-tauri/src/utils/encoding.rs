//! Encoding utilities for converting SHIFT-JIS to UTF-8.
//!
//! Uses encoding_rs crate instead of Windows APIs for portability.

use encoding_rs::Encoding;

/// Decode bytes using the specified encoding label.
/// Defaults to UTF-8 if label is invalid or "UTF-8".
pub fn decode_bytes(data: &[u8], encoding_label: &str) -> String {
    let encoding = Encoding::for_label(encoding_label.as_bytes()).unwrap_or(encoding_rs::UTF_8);
    let (decoded, _, _) = encoding.decode(data);
    decoded.into_owned()
}

/// Convert SHIFT-JIS encoded bytes to UTF-8 string.
/// Kept for backward compatibility.
pub fn shift_jis_to_utf8(data: &[u8]) -> String {
    decode_bytes(data, "SHIFT_JIS")
}

use std::io::{BufRead, BufReader, Read};

/// Read a file with specified encoding and return an iterator over lines.
/// This avoids loading the entire file into memory.
pub fn read_file_lines(file_path: &str, encoding_label: &str) -> std::io::Result<impl Iterator<Item = String>> {
    let file = std::fs::File::open(file_path)?;
    let reader = BufReader::new(file);
    let encoding_label = encoding_label.to_string(); // Clone for closure
    
    Ok(reader.split(b'\n').filter_map(move |line_result| {
        match line_result {
            Ok(bytes) => Some(decode_bytes(&bytes, &encoding_label)),
            Err(_) => None,
        }
    }))
}

/// Read a file with specified encoding and return as UTF-8 string.
pub fn read_file_as_utf8(file_path: &str, encoding_label: &str) -> std::io::Result<String> {
    let mut file = std::fs::File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(decode_bytes(&buffer, encoding_label))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_passthrough() {
        // Pure ASCII/UTF-8 should pass through unchanged
        let data = b"Hello, World!";
        let result = decode_bytes(data, "UTF-8");
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_shift_jis_conversion() {
        // SHIFT-JIS encoding of "日本語" (Japanese text)
        let data: &[u8] = &[0x93, 0xFA, 0x96, 0x7B, 0x8C, 0xEA];
        let result = decode_bytes(data, "SHIFT_JIS");
        assert_eq!(result, "日本語");
    }

    #[test]
    fn test_invalid_encoding_fallback() {
         let data = b"Hello";
         let result = decode_bytes(data, "INVALID_ENCODING_LABEL");
         // Should fallback to UTF-8
         assert_eq!(result, "Hello");
    }
}
