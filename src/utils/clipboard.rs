//! Cross-platform clipboard utilities.
//!
//! Uses arboard crate for clipboard access (works with GNU toolchain).

use arboard::Clipboard;

/// Copy UTF-8 text to the clipboard.
/// 
/// Returns true if successful, false otherwise.
pub fn copy_to_clipboard(text: &str) -> bool {
    // arboard requires a new Clipboard instance for each operation
    match Clipboard::new() {
        Ok(mut clipboard) => {
            match clipboard.set_text(text) {
                Ok(_) => true,
                Err(e) => {
                    eprintln!("Failed to set clipboard text: {}", e);
                    false
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to initialize clipboard: {}", e);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_to_clipboard() {
        // This test modifies the system clipboard, so we just verify it doesn't panic
        let result = copy_to_clipboard("Test content");
        // Result depends on whether we have clipboard access (headless envs might fail)
        let _ = result;
    }
}
