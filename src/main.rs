#![warn(clippy::pedantic, clippy::nursery)]
// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use callback::{
    dynamic_image_to_slint_image, go_to_parent, setimg, update_boxed_image, update_file_tree,
};
use core::f32;
use image::{imageops, DynamicImage, GenericImageView, ImageBuffer};
use logging::setup_logs;
use slint::{ComponentHandle, PhysicalSize, Timer, TimerMode};
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
    let img_ref_for_roll_y = Arc::clone(&img_ref);
    let img_ref_for_roll_x = Arc::clone(&img_ref);

    // TODO: Cleanup
    setimg("./TexturizersLogo.png", &Arc::clone(&img_ref))?;

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

    ui.on_roll_y({
        let ui_handle = ui.as_weak();
        move |y| {
            let mut new;
            {
                let boxed_image = {
                    &mut img_ref_for_roll_y
                        .lock()
                        .expect("Failed to lock mutex")
                        .clone()
                };
                new = *boxed_image.clone();
            }
            new = roll_y(&new, y);
            update_boxed_image(&new, &img_ref_for_roll_y);
            ui_handle
                .unwrap()
                .set_original_image(dynamic_image_to_slint_image(new));
        }
    });

    ui.on_roll_x({
        let ui_handle = ui.as_weak();
        move |x| {
            let mut new;
            {
                let boxed_image = {
                    &mut img_ref_for_roll_x
                        .lock()
                        .expect("Failed to lock mutex")
                        .clone()
                };
                new = *boxed_image.clone();
            }
            new = roll_x(&new, x);
            update_boxed_image(&new, &img_ref_for_roll_x);
            ui_handle
                .unwrap()
                .set_original_image(dynamic_image_to_slint_image(new));
        }
    });

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

    ui.on_setimg({
        let ui_handle = ui.as_weak();
        move |img_path| {
            let ot = Instant::now();

            let ui = ui_handle.unwrap();
            let result = setimg(img_path.as_ref(), &Arc::clone(&img_ref)).unwrap_or_default();

            // Update on the UI
            let st = Instant::now();
            ui.set_original_image(result);
            log::debug!("Time to set: {:#?}", st.elapsed());

            log::warn!("set-img took: {:#?}", ot.elapsed());
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

fn roll_x(img: &DynamicImage, dx: f32) -> DynamicImage {
    if !(-1.0..=1.0).contains(&dx) {
        log::error!("Attempt to roll x by invalid value: {dx}");
        return img.clone();
    }

    if dx.abs() <= f32::EPSILON {
        return img.clone();
    };

    log::debug!("Rolling x by {dx}");

    let (w, h) = img.dimensions();
    if w == 0 {
        return img.clone();
    };
    let dx_pixels = (w as f32 * dx) as i32;
    let dx_pixels = dx_pixels.rem_euclid(w as i32) as u32;

    let left = imageops::crop_imm(img, 0, 0, dx_pixels, h).to_image();
    let right = imageops::crop_imm(img, dx_pixels, 0, w - dx_pixels, h).to_image();

    let mut new_img = ImageBuffer::new(w, h);

    for (x, y, pixel) in right.enumerate_pixels() {
        new_img.put_pixel(x, y, *pixel);
    }

    for (x, y, pixel) in left.enumerate_pixels() {
        new_img.put_pixel(x + w - dx_pixels, y, *pixel);
    }

    DynamicImage::ImageRgba8(new_img)
}

fn roll_y(img: &DynamicImage, dy: f32) -> DynamicImage {
    if !(-1.0..=1.0).contains(&dy) {
        log::error!("Attempt to roll y by invalid value: {dy}");
        return img.clone();
    }

    if dy.abs() <= f32::EPSILON {
        return img.clone();
    };
    log::debug!("Rolling y by {dy}");

    let (w, h) = img.dimensions();
    if h == 0 {
        return img.clone();
    };
    let dy_pixels = (h as f32 * dy) as i32;
    let dy_pixels = dy_pixels.rem_euclid(h as i32) as u32;
    let upper = imageops::crop_imm(img, 0, 0, w, dy_pixels).to_image();
    let lower = imageops::crop_imm(img, 0, dy_pixels, w, h - dy_pixels).to_image();
    let mut new_img = ImageBuffer::new(w, h);

    for (x, y, pixel) in lower.enumerate_pixels() {
        new_img.put_pixel(x, y, *pixel);
    }
    for (x, y, pixel) in upper.enumerate_pixels() {
        new_img.put_pixel(x, y + h - dy_pixels, *pixel);
    }

    DynamicImage::ImageRgba8(new_img)
}
