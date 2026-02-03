//! Windows clipboard utilities.
//!
//! Uses clipboard-win crate for Windows clipboard access.

use clipboard_win::{formats, set_clipboard};

/// Copy UTF-8 text to the Windows clipboard.
/// 
/// Returns true if successful, false otherwise.
pub fn copy_to_clipboard(text: &str) -> bool {
    set_clipboard(formats::Unicode, text).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_to_clipboard() {
        // This test modifies the system clipboard, so we just verify it doesn't panic
        let result = copy_to_clipboard("Test content");
        // Result depends on whether we have clipboard access
        let _ = result;
    }
}
