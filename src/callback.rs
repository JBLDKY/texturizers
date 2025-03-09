#![warn(clippy::pedantic, clippy::nursery)]
// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use image::{DynamicImage, ImageReader};
use slint::{Image, SharedPixelBuffer};

use crate::app::AppWindow;
use crate::{files::update_file_tree, path::update_path};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Handles for moving a directory up
/// NOTE: This modifies the ui
pub fn go_to_parent(ui: &AppWindow) {
    // First we must get the parent dir
    // However, a path may not have a parent dir, in which case we
    // do return the current dir
    let old_path = PathBuf::from(ui.get_path().to_string());
    let new_path = old_path
        .parent()
        .map_or_else(|| old_path.clone(), std::path::Path::to_path_buf);

    // Update the path with the removed current directory version
    update_path(ui, new_path);

    // Globby depends on update path
    update_file_tree(ui);
}

/// Handles changing the displayed image from a path.
/// Updates both the UI **and** our boxed image.
pub fn setimg(
    img_path: &str,
    img_ref: &Arc<Mutex<Box<DynamicImage>>>,
) -> Result<Image, anyhow::Error> {
    //  Read new image from filepath
    let st = Instant::now();
    let dynamic_image = ImageReader::open(img_path)?.decode()?;
    log::debug!("Time to parse into DynamicImage: {:#?}", st.elapsed());

    //  Update our boxed image that we use for in-mem editing
    let st = Instant::now();
    update_boxed_image(&dynamic_image, img_ref);
    log::debug!("Time to box: {:#?}", st.elapsed());

    //  Convert from our library DynamicImage to the format that slint requires
    let st = Instant::now();
    let ui_image = dynamic_image_to_slint_image(dynamic_image);
    log::debug!(
        "Time to convert DynamicImage to slint Image: {:#?}",
        st.elapsed()
    );

    // Done
    Ok(ui_image)
}

#[inline]
fn update_boxed_image(image: &DynamicImage, img_ref: &Arc<Mutex<Box<DynamicImage>>>) {
    let boxed_image = &mut img_ref.lock().expect("Failed to lock mutex");
    **boxed_image = Box::new(image.clone());
}

#[inline]
fn dynamic_image_to_slint_image(image: DynamicImage) -> Image {
    let rgba8 = image.into_rgba8();
    Image::from_rgba8(SharedPixelBuffer::clone_from_slice(
        rgba8.as_raw(),
        rgba8.width(),
        rgba8.height(),
    ))
}
