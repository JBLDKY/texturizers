#![warn(clippy::pedantic, clippy::nursery)]
// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use callback::go_to_parent;
use files::update_file_tree;
use image::{DynamicImage, ImageReader};
use logging::setup_logs;
use slint::Image;
use slint::PhysicalSize;
use slint::{ComponentHandle, Timer, TimerMode};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Instant;
mod app;
mod callback;
mod files;
mod logging;
mod path;
use crate::app::AppWindow;

pub const TIME_TO_INITIALIZE_APP: u64 = 500;
pub const DEFAULT_WIDTH_APP: u32 = 1200;
pub const DEFAULT_HEIGHT_APP: u32 = 800;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logs();
    let ui = AppWindow::new()?;
    ui.window()
        .set_size(PhysicalSize::new(DEFAULT_WIDTH_APP, DEFAULT_HEIGHT_APP));

    let img: Box<DynamicImage> = Box::default();
    let img_ref = Arc::new(Mutex::new(img));
    let img_ref_clone = Arc::clone(&img_ref);
    let img_ref_clone_roll = Arc::clone(&img_ref);

    // Trigger the initial reload
    let timer = Timer::default();
    timer.start(
        TimerMode::SingleShot,
        std::time::Duration::from_millis(TIME_TO_INITIALIZE_APP),
        {
            let ui_handle = ui.as_weak().unwrap();
            move || update_file_tree(&ui_handle)
        },
    );

    ui.on_go_to_parent({
        let ui_handle = ui.as_weak();
        move || {
            log::warn!("go-to-parent");
            let app_window = ui_handle.unwrap();

            // go-to-parent handler
            go_to_parent(&app_window);

            // Return the String path that we set earlier
            // TODO: check if this is necessary
            app_window.get_path()
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

    ui.on_update_file_tree({
        let ui_handle = ui.as_weak();
        move || {
            log::warn!("glob-path");
            update_file_tree(&ui_handle.unwrap());
        }
    });

    // Must be after the callbacks
    ui.run()?;
    Ok(())
}
