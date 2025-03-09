#![warn(clippy::pedantic, clippy::nursery)]
// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use callback::{
    dynamic_image_to_slint_image, go_to_parent, setimg, update_boxed_image, update_file_tree,
};
use core::f32;
use files::list_dir;
use image::{imageops, DynamicImage, GenericImageView, ImageBuffer, ImageReader};
use logging::setup_logs;
use slint::{ComponentHandle, Model, PhysicalSize, Timer, TimerMode, VecModel};
use std::error::Error;
use std::path::PathBuf;
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

#[derive(Debug)]
struct Config {
    x16: bool,
    x32: bool,
    x64: bool,
    x128: bool,
    x256: bool,
    x512: bool,
    w: bool,
    h: bool,
    nearest: bool,
    everything_in_dir: bool,
    format: String,
}

impl Config {
    fn from_raw(raw: &VecModel<bool>) -> Self {
        let iiterator: Vec<bool> = raw.iter().collect();
        let iterator = iiterator.iter();
        log::warn!("{:#?}", iiterator);

        Self {
            x16: iiterator[0],
            x32: iiterator[1],
            x64: iiterator[2],
            x128: iiterator[3],
            x256: iiterator[4],
            x512: iiterator[5],
            w: iiterator[6],
            h: iiterator[7],
            nearest: iiterator[8],
            everything_in_dir: iiterator[9],
            format: "png".to_string(),
        }
    }

    fn nums(&self) -> Vec<usize> {
        let mut res = vec![];

        if self.x16 {
            res.push(16);
        }
        if self.x32 {
            res.push(32);
        }
        if self.x64 {
            res.push(64);
        }
        if self.x128 {
            res.push(128);
        }
        if self.x256 {
            res.push(256);
        }
        if self.x512 {
            res.push(512);
        }
        res
    }

    const fn x128_enabled(&self) -> bool {
        self.x128
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug)]
enum ConfigItem {
    X16,
    X32,
    X64,
    X128,
    X256,
    X512,
    W,
    H,
}

fn main() -> Result<(), Box<dyn Error>> {
    setup_logs();
    let ui = AppWindow::new()?;

    ui.window()
        .set_size(PhysicalSize::new(DEFAULT_WIDTH_APP, DEFAULT_HEIGHT_APP));

    let img: Box<DynamicImage> = Box::default();
    let img_ref = Arc::new(Mutex::new(img));
    let img_ref_for_roll_y = Arc::clone(&img_ref);
    let img_ref_for_roll_x = Arc::clone(&img_ref);
    let img_ref_for_export = Arc::clone(&img_ref);

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

    ui.on_export({
        log::info!("export");
        let ui_handle = ui.as_weak();
        move |nearest, output_dir, name, config| {
            log::info!("export");
            let ui = ui_handle.unwrap();
            let mut parsed_output_dir = PathBuf::from(output_dir.to_string());
            if !parsed_output_dir.is_dir() {
                log::error!(
                    "Output dir {} is not a valid directory",
                    parsed_output_dir.display()
                );
                return;
            }

            let new;
            {
                let boxed_image = {
                    &mut img_ref_for_export
                        .lock()
                        .expect("Failed to lock mutex")
                        .clone()
                };

                new = *boxed_image.clone();
            }

            parsed_output_dir.push(PathBuf::from(name.to_string()));
            log::info!("New name: {}", parsed_output_dir.display());

            let settings = config
                .as_any()
                .downcast_ref::<VecModel<bool>>()
                .expect("Failed to downcast config");

            let config = Config::from_raw(settings);

            let (w, h) = new.dimensions();

            log::info!("{:#?}", config);

            if config.everything_in_dir {
                let files = list_dir(ui.get_path().to_string());
                log::info!("{:#?}", files);

                for file in files {
                    let res = ImageReader::open(file);
                    if res.is_err() {
                        continue;
                    }
                    let dec = res.unwrap().decode();

                    if let Err(_) = dec {
                        continue;
                    }

                    let dec = dec.unwrap();
                    if config.w && w > 0 {
                        for num in config.nums() {
                            let factor = w as usize / num;
                            let resized = dec.resize(
                                num as u32,
                                h * factor as u32,
                                imageops::FilterType::Nearest,
                            );
                            let suffix = format!(
                                "{}_w_x{}_w{}_h{}.{}",
                                name,
                                num,
                                resized.width(),
                                resized.height(),
                                config.format
                            );

                            let mut new_name = parsed_output_dir.clone();
                            new_name.push(suffix);

                            log::debug!("Saving (w): {}", new_name.display());
                            resized.save(new_name);
                        }
                    }

                    log::debug!("config.h: {}, h: {}", config.h, h);
                    if config.h && h > 0 {
                        for num in config.nums() {
                            let factor = h as usize / num;
                            let resized = dec.resize(
                                w * factor as u32,
                                num as u32,
                                imageops::FilterType::Nearest,
                            );
                            let suffix = format!(
                                "{}_w_x{}_w{}_h{}.{}",
                                name,
                                num,
                                resized.width(),
                                resized.height(),
                                config.format
                            );

                            let mut new_name = parsed_output_dir.clone();
                            new_name.push(suffix);

                            log::debug!("Saving (h): {}", new_name.display());
                            resized.save(new_name);
                        }
                    }
                }
                return;
            }

            if config.w && w > 0 {
                for num in config.nums() {
                    let factor = w as usize / num;
                    let resized =
                        new.resize(num as u32, h * factor as u32, imageops::FilterType::Nearest);
                    let suffix = format!(
                        "{}_w_x{}_w{}_h{}.{}",
                        name,
                        num,
                        resized.width(),
                        resized.height(),
                        config.format
                    );

                    let mut new_name = parsed_output_dir.clone();
                    new_name.push(suffix);

                    log::debug!("Saving (w): {}", new_name.display());
                    resized.save(new_name);
                }
            }

            log::debug!("config.h: {}, h: {}", config.h, h);
            if config.h && h > 0 {
                for num in config.nums() {
                    let factor = h as usize / num;
                    let resized =
                        new.resize(w * factor as u32, num as u32, imageops::FilterType::Nearest);
                    let suffix = format!(
                        "{}_w_x{}_w{}_h{}.{}",
                        name,
                        num,
                        resized.width(),
                        resized.height(),
                        config.format
                    );

                    let mut new_name = parsed_output_dir.clone();
                    new_name.push(suffix);

                    log::debug!("Saving (h): {}", new_name.display());
                    resized.save(new_name);
                }
            }
        }
    });

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

    ui.on_update_file_tree({
        let ui_handle = ui.as_weak();
        move || {
            log::warn!("glob-path");
            update_file_tree(&ui_handle.unwrap());
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
            ui.set_image_path(img_path.clone());

            let ui = ui_handle.unwrap();
            let result = setimg(img_path.as_ref(), &Arc::clone(&img_ref)).unwrap_or_default();

            // Update on the UI
            let st = Instant::now();
            ui.set_original_image(result);
            log::debug!("Time to set: {:#?}", st.elapsed());

            log::warn!("set-img took: {:#?}", ot.elapsed());
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
