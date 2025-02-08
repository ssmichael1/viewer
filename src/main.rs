// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod gui;
mod imgproc;

use camera::CameraError;
use camera::CameraFrame;

use imgproc::ImageQueue;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Create a GUI
    let mut thegui = gui::Gui::new()?;

    // create an image processing chain
    let imgproc = imgproc::ImageProcessor::new();
    // set parameter structure for image processor
    imgproc.lock().unwrap().set_params(thegui.get_params());

    // Tell the chain to call the gui processor when it is complete
    imgproc.lock().unwrap().set_sink(thegui.on_processed());

    // Image queue: creates a separate thread to process frames
    let imgqueue = ImageQueue::new();
    // Process images whenever a frame arrives
    let pclone = imgproc.clone();
    // Start the image queue (creates a thread)
    imgqueue.start(move |frame: CameraFrame| pclone.lock().unwrap().process_frame(frame));

    let cameras = camera::get_available_cameras();
    if cameras.is_empty() {
        eprintln!("No cameras found");
        return Ok(());
    }
    println!("Found {} cameras", cameras.len());
    cameras.iter().for_each(|c| println!("{}", c.name));

    println!("Found camera {}", cameras.first().unwrap().name);
    let mut cam0 = (cameras.last().unwrap().get_camera)();
    //cam0.connect()?;

    let _ = cam0.set_frame_callback(Box::new(move |frame| -> Result<(), CameraError> {
        imgqueue.add_frame_to_queue(frame);
        Ok(())
    }));
    cam0.start()?;

    thegui.run()?;

    cam0.stop()?;
    cam0.disconnect()?;

    Ok(())
}
