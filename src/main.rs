#![warn(clippy::pedantic, clippy::nursery)]
// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use anyhow::anyhow;
use env_logger::Builder;
use glob::glob;
use log::LevelFilter;
use slint::{ComponentHandle, Weak};
use slint::{Model, PhysicalSize, VecModel};
use slint::{Timer, TimerMode};
use std::path::Path;
use std::{error::Error, path::PathBuf};

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    setup_logging();
    let ui = AppWindow::new()?;

    let timer = Timer::default();
    timer.start(
        TimerMode::SingleShot,
        std::time::Duration::from_millis(500),
        {
            let ui_handle = ui.as_weak();
            move || globby(&ui_handle)
        },
    );
    ui.window().set_size(PhysicalSize::new(1200, 800));

    ui.on_go_to_parent({
        let ui_handle = ui.as_weak();
        move || {
            log::warn!("go-to-parent");
            let ui = ui_handle.unwrap();

            let old_path = PathBuf::from(ui.get_path().to_string());
            let new_path = old_path
                .parent()
                .map_or_else(|| old_path.clone(), std::path::Path::to_path_buf);

            let res = new_path.to_str().unwrap_or_default();
            ui.set_path(res.into());
            globby(&ui_handle);
            res.into()
        }
    });

    ui.on_setimg({
        let ui_handle = ui.as_weak();
        move |title| {
            log::warn!("set-img");
            let ui = ui_handle.unwrap();

            let mut img_path = ui.get_path().to_string();
            img_path.push_str(title.as_ref());
            let img = image::open(&img_path);
            if img.is_err() {
                log::error!("Error opening {}", img_path);
                return;
            }
            let unwrapped = img.unwrap().into_rgba8();
            let real = {
                slint::Image::from_rgba8(slint::SharedPixelBuffer::clone_from_slice(
                    unwrapped.as_raw(),
                    unwrapped.width(),
                    unwrapped.height(),
                ))
            };
            ui.set_original_image(real);
            log::warn!("Loaded: {}", img_path);
        }
    });

    ui.on_glob_path({
        let ui_handle = ui.as_weak();
        move || {
            log::warn!("glob-path");
            globby(&ui_handle);
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

fn globby(ui_handle: &Weak<AppWindow>) {
    let ui = ui_handle.unwrap();

    let mut old_path = ui.get_path().to_string();
    log::warn!("Globbing: {}", &old_path);

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
            log::debug!("found dir: {}", filename.display());
            todos_vec.push(TodoItem {
                title: filename
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    .into(),
                checked: false,
                dir: filename.is_dir(),
                full_path: filename.to_str().unwrap_or_default().into(),
            });
        }
    }

    for filename in list_dir(ui.get_path().to_string()) {
        if filename.is_file() {
            log::debug!("found file: {}", filename.display());
            todos_vec.push(TodoItem {
                title: filename
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    .into(),
                checked: false,
                dir: filename.is_dir(),
                full_path: filename.to_str().unwrap_or_default().into(),
            });
        }
    }
}
