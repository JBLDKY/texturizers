#![warn(clippy::pedantic, clippy::nursery)]
// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use crate::app::{AppWindow, TodoItem};
use crate::path::maybe_add_character;
use anyhow::anyhow;
use glob::glob;
use slint::{Model, VecModel};
use std::path::Path;
use std::path::PathBuf;

pub fn update_file_tree(ui: &AppWindow) {
    // Sort the files in the current UI path directory
    // If the ui path changed, this must be called AFTER said change
    let files = sort_filetree(list_dir(ui.get_path().into()));

    // Get a ref to the filetre model
    let filetree = ui.get_todo_model();
    let filetree = filetree
        .as_any()
        .downcast_ref::<VecModel<TodoItem>>()
        .expect("The ui has a VecModel; the list of images");

    // Empty the current items
    filetree.clear();

    // Populate with new items
    for file in files {
        filetree.push(filetree_entry_from_path(file));
    }
}

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

fn file_name_format(path: impl AsRef<Path>) -> String {
    path.as_ref()
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_string()
}

fn filetree_entry_name_from_path(path: impl AsRef<Path>) -> String {
    if path.as_ref().is_dir() {
        return dir_name_format(path);
    }

    file_name_format(path)
}

fn filetree_entry_from_path(path: impl AsRef<Path>) -> TodoItem {
    let file = path.as_ref();
    TodoItem {
        title: filetree_entry_name_from_path(file).into(),
        checked: false,
        is_dir: file.is_dir(),
        full_path: file.to_str().unwrap_or_default().into(),
    }
}

/// Sorts a list of files by directories first
fn sort_filetree(mut files: Vec<PathBuf>) -> Vec<PathBuf> {
    files.sort_by_key(|file| file.is_file());
    files
}

/// Creates a globstring from a path
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

/// Get all files in a directory
/// Returns an empty list if something goes wrong
fn list_dir(path: String) -> Vec<PathBuf> {
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
