#![warn(clippy::pedantic, clippy::nursery)]
// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use crate::app::FileItem;
use crate::path::maybe_add_character;
use anyhow::anyhow;
use glob::glob;
use std::path::Path;
use std::path::PathBuf;

/// Constructs a button entry for the filetree from a path
pub fn filetree_entry_from_path(path: impl AsRef<Path>) -> FileItem {
    let file = path.as_ref();
    FileItem {
        title: filetree_entry_name_from_path(file).into(),
        checked: false,
        is_dir: file.is_dir(),
        full_path: file.to_str().unwrap_or_default().into(),
    }
}

/// Sorts a list of files by directories first
pub fn sort_filetree(mut files: Vec<PathBuf>) -> Vec<PathBuf> {
    files.sort_by_key(|file| file.is_file());
    files
}

/// Get all files in a directory
/// Returns an empty list; if something goes wrong
pub fn list_dir(path: String) -> Vec<PathBuf> {
    let path = match glob_string_from_path(path) {
        Ok(v) => v,
        Err(e) => {
            log::error!("{}", e);
            return vec![];
        }
    };
    let globbed = match glob(&path) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Could not glob files from your system. Error: {e}");
            return vec![];
        }
    };

    globbed.filter_map(std::result::Result::ok).collect()
}

/// Constructs the text we display for directories
fn dir_name_format(path: impl AsRef<Path>) -> String {
    format!(
        "> {}/",
        path.as_ref()
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
    )
}

/// Constructs the text we display for files
fn file_name_format(path: impl AsRef<Path>) -> String {
    path.as_ref()
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_string()
}

/// Constructs the text we display in the filetree buttons from a path
fn filetree_entry_name_from_path(path: impl AsRef<Path>) -> String {
    if path.as_ref().is_dir() {
        return dir_name_format(path);
    }

    file_name_format(path)
}

/// Creates a globstring from a path to get all items in a directory
fn glob_string_from_path(path: impl AsRef<Path>) -> Result<String, anyhow::Error> {
    let mut pathbuf = path.as_ref().to_path_buf();

    if !pathbuf.is_file() && !pathbuf.is_dir() && !pathbuf.exists() {
        let msg = format!("Entered path is not a dir or file: {}", pathbuf.display());
        log::error!("{}", msg);
        return Err(anyhow!(msg));
    }

    if pathbuf.is_file() {
        pathbuf = pathbuf.parent().unwrap_or_else(|| Path::new("")).into();
    }

    let mut path: String = pathbuf.to_string_lossy().to_string();
    path = maybe_add_character(maybe_add_character(path, '/'), '*');

    Ok(path)
}
