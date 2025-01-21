// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cameraframe;
mod colormap;
mod gui;
mod imgproc;
mod simsource;

pub use cameraframe::CameraFrame;
use imgproc::ImageQueue;
use simsource::SimSource;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Create a GUI
    let mut thegui = gui::Gui::new()?;

    // create an image processing chain
    let imgproc = imgproc::ImageProcessor::<u16>::new();
    // set parameter structure for image processor
    imgproc.lock().unwrap().set_params(thegui.get_params());
    // Tell the chain to call the gui processor when it is complete
    imgproc
        .lock()
        .unwrap()
        .set_sink(thegui.processed_callback::<u16>());

    // Image queue: creates a separate thread to process frames
    let imgqueue = ImageQueue::<u16>::new();
    // Process images whenever a frame arrives
    let pclone = imgproc.clone();
    // Start the image queue (creates a thread)
    imgqueue.start(move |frame: CameraFrame<u16>| {
        pclone.lock().unwrap().process_frame(frame);
    });

    // Simulated camera source
    let mut simsource = SimSource::new();
    // Dump frames into image queue when they are ready
    simsource.start(move |frame: CameraFrame<u16>| imgqueue.on_frame_available(frame));

    thegui.run()?;

    Ok(())
}
