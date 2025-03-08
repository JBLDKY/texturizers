#![warn(clippy::pedantic, clippy::nursery)]
// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use anyhow::anyhow;
use env_logger::Builder;
use glob::glob;
use log::LevelFilter;
use slint::platform::{Renderer, WindowAdapter};
use slint::{ComponentHandle, Window};
use slint::{Model, PhysicalSize, VecModel};
use slint_generatedAppWindow::InnerLineEditBase_root_1;
use std::path::Path;
use std::{error::Error, path::PathBuf};

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    setup_logging();
    let ui = AppWindow::new()?;

    ui.window().set_size(PhysicalSize::new(600, 800));
    ui.on_glob_path({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();

            let mut old_path = ui.get_path().to_string();

            if !old_path.ends_with('/') {
                old_path.push('/');
            }

            ui.set_path(old_path.into());
            log::info!("User entered new path: {}", ui.get_path());

            let todos = ui.get_todo_model();
            let todos_vec = todos
                .as_any()
                .downcast_ref::<VecModel<TodoItem>>()
                .expect("The ui has a VecModel; the list of images");

            todos_vec.clear();

            for filename in list_dir(ui.get_path().to_string()) {
                if filename.is_dir() {
                    todos_vec.push(TodoItem {
                        title: filename
                            .file_name()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default()
                            .into(),
                        checked: false,
                        dir: filename.is_dir(),
                    });
                }
            }

            for filename in list_dir(ui.get_path().to_string()) {
                if filename.is_file() {
                    todos_vec.push(TodoItem {
                        title: filename
                            .file_name()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default()
                            .into(),
                        checked: false,
                        dir: filename.is_dir(),
                    });
                }
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

// Parses a path to a valid path
fn parse_path(mut path: String) -> Result<String, anyhow::Error> {
    let pathbuf = PathBuf::from(&path);
    if !pathbuf.is_file() && !pathbuf.is_dir() && !pathbuf.exists() {
        let msg = format!("Entered path is not a dir or file: {}", pathbuf.display());
        log::error!("{}", msg);
        return Err(anyhow!(msg));
    }

    if pathbuf.is_file() {
        pathbuf
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_str()
            .unwrap_or_default()
            .clone_into(&mut path);
    }

    if !path.ends_with('/') {
        path.push('/');
    }
    path.push('*');

    Ok(path)
}
/// Get all filers in a directory
/// Returns an empty list if something goes wrong
fn list_dir(path: String) -> Vec<PathBuf> {
    let path = match parse_path(path) {
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
