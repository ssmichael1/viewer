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

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Create a GUI
    let mut thegui = gui::Gui::new()?;

    // create an image processing chain
    let imgproc = imgproc::ImageProcessor::<u16>::new();
    // set parameter structure for image processor
    imgproc.lock().unwrap().set_params(thegui.get_params());

    // Tell the chain to call the gui processor when it is complete
    imgproc.lock().unwrap().set_sink(thegui.on_processed());

    // Image queue: creates a separate thread to process frames
    let imgqueue = ImageQueue::<u16>::new();
    // Process images whenever a frame arrives
    let pclone = imgproc.clone();
    // Start the image queue (creates a thread)
    imgqueue.start(move |frame: CameraFrame<u16>| {
        pclone.lock().unwrap().process_frame(frame);
    });

    let cameras = svbony::get_connected_cameras().unwrap_or_default();
    println!("Found {} cameras", cameras.len());
    for cam in cameras.clone() {
        println!("Camera: {:?}", cam);
    }

    // Simulated camera source
    let mut cam: Option<svbony::SVBonyCamera> = None;
    if cameras.is_empty() {
        let mut sim = SimSource::new();
        sim.start(move |frame: CameraFrame<u16>| imgqueue.on_frame_available(frame));
    } else {
        println!("SDK Version = {}", svbony::lowlevel::get_sdk_version());
        println!("camera = {:?}", cameras.first().unwrap().camera_id);
        cam = Some(svbony::SVBonyCamera::new(0)?);
        println!("got to here");
        let width = cam.as_ref().unwrap().max_width();
        let height = cam.as_ref().unwrap().max_height();
        cam.as_mut().unwrap().set_exposure(1000)?;
        cam.as_mut().unwrap().set_gain(50)?;
        cam.as_mut()
            .unwrap()
            .set_control_value(svbony::SVBControlType::SVBFrameSpeedMode, 0)?;

        cam.as_mut()
            .unwrap()
            .add_function_callback(move |data: &svbony::FrameData<'_>, ts| {
                if let svbony::FrameData::U16Frame(d) = data {
                    let frame = CameraFrame::<u16>::create(
                        0.1,
                        ts,
                        12,
                        crate::cameraframe::FrameData::<u16> {
                            width: width as u32,
                            height: height as u32,
                            data: d.to_vec().iter().map(|x| x.swap_bytes() >> 4).collect(),
                        },
                    );
                    imgqueue.on_frame_available(frame);
                }
            })?;

        let mut c = cam.clone();
        std::thread::spawn(move || {
            let _ = c.as_mut().unwrap().run();
        });
    }
    // Dump frames into image queue when they are ready

    thegui.run()?;
    if let Some(c) = cam.as_mut() {
        c.stop();
        c.close_camera()?;
    }

    Ok(())
}
