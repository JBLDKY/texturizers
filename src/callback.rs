#![warn(clippy::pedantic, clippy::nursery)]
// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use crate::app::AppWindow;
use crate::{files::update_file_tree, path::update_path};
use std::path::PathBuf;

/// Handles for moving a directory up
/// NOTE: This modifies the ui
pub fn go_to_parent(ui: &AppWindow) {
    // First we must get the parent dir
    // However, a path may not have a parent dir, in which case we
    // do return the current dir
    let old_path = PathBuf::from(ui.get_path().to_string());
    let new_path = old_path
        .parent()
        .map_or_else(|| old_path.clone(), std::path::Path::to_path_buf);

    // Update the path with the removed current directory version
    update_path(ui, new_path);

    // Globby depends on update path
    update_file_tree(ui);
}
