#![warn(clippy::pedantic, clippy::nursery)]
// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use callback::{
    dynamic_image_to_slint_image, go_to_parent, setimg, update_boxed_image, update_file_tree,
};
use image::{imageops, DynamicImage, GenericImage, GenericImageView, ImageBuffer};
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
    let img_ref_for_manip = Arc::clone(&img_ref);
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
    // def roll_y(im: PIL.Image, dy: float) -> PIL.Image:
    //     if 0 > dy > 1:
    //         raise ValueError(f"Value by which to roll Y is outside of 1 and 0: {dy}")
    //
    //     w, h = im.size
    //
    //     dy = trunc(h * dy)
    //     dy %= w
    //
    //     upper = im.crop((0, 0, w, dy))
    //     lower = im.crop((0, dy, w, h))
    //     im.paste(upper, (0, h - dy, w, h))
    //     im.paste(lower, (0, 0, w, h - dy))
    //
    //     return im
    //
    //
    // def roll_x(im: PIL.Image, dx: float) -> PIL.Image:
    //     if 0 > dx > 1:
    //         raise ValueError(f"Value by which to roll X is outside of 1 and 0: {dx}")
    //
    //     w, h = im.size
    //
    //     dx = trunc(w * dx)
    //     dx %= w
    //
    //     left = im.crop((0, 0, dx, h))
    //     right = im.crop((dx, 0, w, h))
    //     im.paste(left, (w - dx, 0, w, h))
    //     im.paste(right, (0, 0, w - dx, h))
    //
    //     return im

    ui.on_roll_image({
        let ui_handle = ui.as_weak();
        move |start, end| {
            log::warn!("roll-image");
            log::debug!("start: {start:#?}");
            log::debug!("end: {end:#?}");
            let ui = ui_handle.unwrap();

            let mut new;
            {
                let boxed_image = {
                    &mut img_ref_for_manip
                        .lock()
                        .expect("Failed to lock mutex")
                        .clone()
                };
                new = *boxed_image.clone();
            }

            let w = new.width() as i32;
            let h = new.height() as i32;

            let travelled_x = (end.x - start.x) % w;
            let pdx = travelled_x as f32 / w as f32;

            let travelled_y = (end.y - start.y) % h;
            let pdy = travelled_y as f32 / w as f32;

            new = roll_x(new, pdx);
            new = roll_y(new, pdy);

            update_boxed_image(&new, &img_ref_for_manip);
            ui.set_original_image(dynamic_image_to_slint_image(new));
            // log::debug!("Replacing");
            // imageops::replace(&mut new, &left, dx.into(), dy.into());
            // imageops::replace(&mut new, &right, dx.into(), dy.into());
            // imageops::replace(&mut new, &left, end.x.into(), end.y.into());
            // imageops::flip_horizontal_in_place(&mut new);
            // log::debug!("done");

            //     left = im.crop((0, 0, dx, h))
            // let mut dyn_img = inner.lock().unwrap();
            // let new_img = dyn_img.flipv();
            // *dyn_img = Box::new(new_img.clone());
            // drop(dyn_img);
            //
            // let unwrapped = new_img.into_rgba8();
            // let real = {
            //     slint::Image::from_rgba8(slint::SharedPixelBuffer::clone_from_slice(
            //         unwrapped.as_raw(),
            //         unwrapped.width(),
            //         unwrapped.height(),
            //     ))
            // };
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

fn roll_x(mut img: DynamicImage, dx: f32) -> DynamicImage {
    assert!(
        !(dx < -1.0 || dx > 1.0),
        "Value by which to roll X is outside of 1 and 0: {dx}"
    );

    log::info!("Rolling x by: {dx}");

    let (w, h) = img.dimensions();
    let dx_pixels = (w as f32 * dx) as u32;
    let dx_pixels = dx_pixels % w;

    let left = imageops::crop_imm(&img, 0, 0, dx_pixels, h).to_image();
    let right = imageops::crop_imm(&img, dx_pixels, 0, w - dx_pixels, h).to_image();

    let mut new_img = ImageBuffer::new(w, h);

    for (x, y, pixel) in right.enumerate_pixels() {
        new_img.put_pixel(x, y, *pixel);
    }
    for (x, y, pixel) in left.enumerate_pixels() {
        new_img.put_pixel(x + w - dx_pixels, y, *pixel);
    }

    DynamicImage::ImageRgba8(new_img)
}

fn roll_y(mut img: DynamicImage, dy: f32) -> DynamicImage {
    if dy < -1.0 || dy > 1.0 {
        panic!("Value by which to roll Y is outside of 1 and 0: {}", dy);
    }

    log::info!("Rolling y by: {dy}");

    let (w, h) = img.dimensions();
    let dy_pixels = (h as f32 * dy) as u32;
    let dy_pixels = dy_pixels % h;

    let upper = imageops::crop_imm(&img, 0, 0, w, dy_pixels).to_image();
    let lower = imageops::crop_imm(&img, 0, dy_pixels, w, h - dy_pixels).to_image();

    let mut new_img = ImageBuffer::new(w, h);

    for (x, y, pixel) in lower.enumerate_pixels() {
        new_img.put_pixel(x, y, *pixel);
    }

    for (x, y, pixel) in upper.enumerate_pixels() {
        new_img.put_pixel(x, y + h - dy_pixels, *pixel);
    }

    DynamicImage::ImageRgba8(new_img)
}
