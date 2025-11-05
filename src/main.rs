// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod gui;
mod imgproc;

use camera::prelude::*;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    tracing_subscriber::fmt::init();

    // Create a GUI
    let mut thegui = gui::Gui::new()?;

    // create an image processing chain
    let imgproc = imgproc::ImageProcessor::new();
    // set parameter structure for image processor
    imgproc.lock().unwrap().set_params(thegui.get_params());

    // Tell the chain to call the gui processor when it is complete
    imgproc.lock().unwrap().set_sink(thegui.on_processed());

    let cameras = get_connected_cameras();
    if cameras.is_empty() {
        eprintln!("No cameras found");
        return Ok(());
    }
    cameras.iter().for_each(|c| println!("{}", c.name()));

    let list: Vec<String> = cameras.iter().map(|c| c.name().to_string()).collect();
    thegui.set_camera_list(list);
    thegui.set_camera(cameras.last().unwrap().clone());

    thegui
        .run()
        .unwrap_or_else(|e| eprintln!("Error running GUI: {}", e));

    Ok(())
}
