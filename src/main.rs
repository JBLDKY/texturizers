#![warn(clippy::pedantic, clippy::nursery)]
// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use anyhow::anyhow;
use env_logger::Builder;
use glob::glob;
use image::{DynamicImage, ImageReader};
use log::LevelFilter;
use slint::{ComponentHandle, Image, Weak};
use slint::{Model, PhysicalSize, VecModel};
use slint::{Timer, TimerMode};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::{error::Error, path::PathBuf};

slint::include_modules!();
pub const TIME_TO_INITIALIZE_APP: u64 = 500;
pub const DEFAULT_WIDTH_APP: u32 = 1200;
pub const DEFAULT_HEIGHT_APP: u32 = 800;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logging();
    let ui = AppWindow::new()?;
    let img: Box<DynamicImage> = Box::default();
    let img_ref = Arc::new(Mutex::new(img));
    let img_ref_clone = Arc::clone(&img_ref);
    let img_ref_clone_roll = Arc::clone(&img_ref);

    let timer = Timer::default();
    timer.start(
        TimerMode::SingleShot,
        std::time::Duration::from_millis(TIME_TO_INITIALIZE_APP),
        {
            let ui_handle = ui.as_weak();
            move || globby(&ui_handle)
        },
    );
    ui.window()
        .set_size(PhysicalSize::new(DEFAULT_WIDTH_APP, DEFAULT_HEIGHT_APP));

    ui.on_go_to_parent({
        let ui_handle = ui.as_weak();
        move || {
            log::warn!("go-to-parent");
            let ui = ui_handle.unwrap();

            let old_path = PathBuf::from(ui.get_path().to_string());
            let new_path = old_path
                .parent()
                .map_or_else(|| old_path.clone(), std::path::Path::to_path_buf);

            let res = maybe_add_character(new_path.to_str().unwrap_or_default().to_string(), '/');

            ui.set_path(res.clone().into());
            globby(&ui_handle);
            res.into()
        }
    });

    ui.on_roll_image({
        let ui_handle = ui.as_weak();
        move |_| {
            log::warn!("roll-image");
            let ui = ui_handle.unwrap();

            {
                let inner = &img_ref_clone_roll;
                let mut dyn_img = inner.lock().unwrap();
                let new_img = dyn_img.flipv();
                *dyn_img = Box::new(new_img.clone());
                drop(dyn_img);

                let unwrapped = new_img.into_rgba8();
                let real = {
                    slint::Image::from_rgba8(slint::SharedPixelBuffer::clone_from_slice(
                        unwrapped.as_raw(),
                        unwrapped.width(),
                        unwrapped.height(),
                    ))
                };
                ui.set_original_image(real);
            }
        }
    });

    ui.on_setimg({
        let ui_handle = ui.as_weak();
        move |img_path| {
            log::warn!("set-img");
            let ui = ui_handle.unwrap();

            // let mut img_path = ui.get_path().to_string();
            // img_path.push_str(title.as_ref());
            let st = Instant::now();

            let img = ImageReader::open(&img_path).unwrap().decode();

            log::info!("Time to open: {:#?}", st.elapsed());
            if img.is_err() {
                log::error!("Error opening {}", img_path);
                return Image::default();
            }

            let img_copy = img.unwrap();
            {
                let inner = &img_ref_clone;
                let mut dyn_img = inner.lock().unwrap();
                *dyn_img = Box::new(img_copy.clone());
            }

            let unwrapped = img_copy.into_rgba8();
            log::info!("Time to into_rgba8: {:#?}", st.elapsed());
            let real = {
                slint::Image::from_rgba8(slint::SharedPixelBuffer::clone_from_slice(
                    unwrapped.as_raw(),
                    unwrapped.width(),
                    unwrapped.height(),
                ))
            };
            log::info!("Time to clone: {:#?}", st.elapsed());
            ui.set_original_image(real.clone());
            log::info!("Time to set: {:#?}", st.elapsed());
            log::warn!("Loaded: {}", img_path);
            real
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

    path = maybe_add_character(maybe_add_character(path, '/'), '*');

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

    old_path = maybe_add_character(old_path, '/');

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
            let name = format!(
                "> {}/",
                filename
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
            );

            todos_vec.push(TodoItem {
                title: name.into(),
                checked: false,
                is_dir: filename.is_dir(),
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
                is_dir: filename.is_dir(),
                full_path: filename.to_str().unwrap_or_default().into(),
            });
        }
    }
}

#[inline]
fn maybe_add_character(mut string: String, character: char) -> String {
    if !string.ends_with(character) {
        string.push(character);
    }
    string
}
