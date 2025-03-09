#![warn(clippy::pedantic, clippy::nursery)]
// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use crate::app::AppWindow;
use std::path::Path;

pub fn update_path(ui: &AppWindow, path: impl AsRef<Path>) {
    // Parse the path to a string
    let parsed = path.as_ref().to_str().unwrap_or_default().to_string();

    // Append '/' if it does not yet end with '/' just to be consistent
    let with_forward_slash = maybe_add_character(parsed, '/');

    // Update the source of truth path
    ui.set_path(with_forward_slash.into());
}

#[inline]
pub fn maybe_add_character(mut string: String, character: char) -> String {
    if !string.ends_with(character) {
        string.push(character);
    }
    string
}
