#![warn(clippy::pedantic, clippy::nursery)]
// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use slint::{slint, Model, ModelRc, SharedString, VecModel};
use std::{
    borrow::{BorrowMut, Cow},
    error::Error,
    ffi::OsStr,
    fs::File,
    ops::Deref,
    path::{Path, PathBuf},
    rc::Rc,
};

use anyhow::anyhow;
use env_logger::Builder;
use glob::glob;
use home::home_dir;
use log::LevelFilter;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    setup_logging();
    let ui = AppWindow::new()?;

    let ui_handle = ui.as_weak();
    ui.on_glob_path({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_path(ui.get_path());
            log::info!("User entered new path: {}", ui.get_path());

            let todos = ui.get_todo_model();
            let todos_vec = todos
                .as_any()
                .downcast_ref::<VecModel<TodoItem>>()
                .expect("The ui has a VecModel; the list of images");

            todos_vec.clear();

            for filename in list_dir(ui.get_path().to_string()) {
                todos_vec.push(TodoItem {
                    title: filename.into(),
                    checked: false,
                });
            }
        }
    });

    // Must be after the callbacks
    ui.run()?;
    Ok(())
}

fn setup_logging() {
    // let app_dir = home_dir().ok_or_else(|| anyhow!("Cannot find home directory"))?;

    Builder::new()
        // .target(env_logger::Target::Pipe(Box::new(log_file)))
        .filter_level(LevelFilter::Debug)
        .init();
}

fn list_dir(mut path: String) -> Vec<String> {
    if !PathBuf::from(&path).is_dir() {
        log::error!("Entered path is not a directory");
    }

    if !path.ends_with('/') {
        path.push('/');
    }
    path.push('*');

    let globbed = match glob(&path) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Could not glob files from your system. Error: {e}");
            return vec![];
        }
    };

    globbed
        .filter_map(std::result::Result::ok)
        .map(|path| {
            path.file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
                .to_string()
        })
        .collect()
}
