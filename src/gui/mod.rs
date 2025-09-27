use crate::imgproc::ImageProcessor;
use crate::imgproc::ProcResult;
use std::error::Error;

use slint::Image;
use slint::Model;
use slint::Rgba8Pixel;
use slint::SharedPixelBuffer;

slint::include_modules!();

use camera::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};

use anyhow::Result;

#[derive(Clone, Debug, Copy)]
pub enum FCScaleType {
    Manual,
    Auto,
    Max,
}

#[derive(Clone, Debug)]
pub struct GuiParams {
    pub gamma: f64,
    pub fcscaletype: FCScaleType,
    pub scale_range: (i32, i32),
    pub colorscale: String,
}

impl Default for GuiParams {
    fn default() -> Self {
        Self {
            gamma: 1.0,
            fcscaletype: FCScaleType::Max,
            scale_range: (0, 65535),
            colorscale: "grayscale".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Gui {
    pub ui: Rc<RefCell<AppWindow>>,
    pub params: Arc<RwLock<GuiParams>>,
    pub procresult: Arc<RwLock<ProcResult>>,
    pub camera: Option<Camera>,
    pub proc: Arc<Mutex<ImageProcessor>>,
}

impl Gui {
    pub fn get_params(&self) -> Arc<RwLock<GuiParams>> {
        self.params.clone()
    }

    pub fn set_camera_list(&self, list: Vec<String>) {
        self.ui
            .borrow()
            .global::<Shared>()
            .set_camera_list(slint::VecModel::from_slice(
                &list
                    .iter()
                    .map(|s| slint::SharedString::from(s.as_str()))
                    .collect::<Vec<_>>(),
            ));
    }

    pub fn set_camera(&mut self, mut cam: Camera) {
        self.camera = Some(cam.clone());

        let binding = self.ui.borrow();
        let globals = binding.global::<Shared>();
        match &cam {
            Camera::SVBony(c) => {
                globals.set_camera_type(CameraType::SVBony);
                let c = c.read().unwrap();
                c.capabilities.iter().for_each(|cap| {
                    println!("  Capability: {}", cap);
                    let val = c
                        .get_control_value(cap.control_type)
                        .map_err(|e| {
                            println!("Failed to get control value for {}: {}", cap.name, e);
                            e
                        })
                        .unwrap_or(30);
                    println!("    Value: {}", val);
                });
                globals.set_svbony_capabilities(slint::VecModel::from_slice(
                    &c.capabilities
                        .iter()
                        .map(|cap| {
                            let mut maxval = cap.max_value;
                            if cap.control_type
                                == camera::svbony::lowlevel::SVBControlType::SVBExposure
                            {
                                maxval = 100000; // Limit to 100 milliseconds
                            }
                            SVBonyCapDisp {
                                name: slint::SharedString::from(cap.name.as_str()),
                                description: slint::SharedString::from(cap.description.as_str()),
                                minval: cap.min_value,
                                maxval,
                                default: cap.default_value,
                                auto: cap.is_auto_supported,
                                writeable: cap.is_writeable,
                                value: c.get_control_value(cap.control_type).unwrap_or(30),
                            }
                        })
                        .collect::<Vec<SVBonyCapDisp>>(),
                ));
            }
            Camera::Sim(_c) => {
                //let c = c.read().unwrap();
                globals.set_camera_type(CameraType::Sim);
            }
        }

        self.proc.lock().unwrap().set_params(self.get_params());
        self.proc.lock().unwrap().set_sink(self.on_processed());
        self.camera = Some(cam.clone());
        let gui = self.clone();

        gui.ui.borrow().global::<Shared>().on_svbony_cap_changed({
            let cam = gui.camera.clone();
            println!("cam = {:?}", cam.is_some());
            move |name: slint::SharedString, value: i32| {
                println!("Capability change: {} -> {}", name, value);
                if let Some(Camera::SVBony(c)) = &cam {
                    println!("found the camera");
                    let c = c.write().unwrap();
                    if let Some(cap) = c.capabilities.iter().find(|cap| cap.name == name.as_str()) {
                        println!("Found capability: {} -> {}", cap.name, value);
                        if cap.is_writeable {
                            let _ =
                                c.set_control_value(cap.control_type, value, false)
                                    .map_err(|e| {
                                        println!(
                                            "Failed to set control value {:?} to {} : {}",
                                            cap.control_type, value, e
                                        );
                                        e
                                    });
                        }
                    }
                }
            }
        });

        let _ = cam.set_frame_callback(move |frame: &CameraFrame| -> Result<(), CameraError> {
            gui.proc
                .lock()
                .unwrap()
                .process_frame(&std::sync::Arc::new(frame.clone()));
            Ok(())
        });

        cam.connect().unwrap();
        cam.start().unwrap();
    }

    pub fn on_processed(&self) -> Box<dyn Fn(&ProcResult) + Send + 'static> {
        let ui_handle: slint::Weak<AppWindow> = self.ui.borrow().as_weak();
        let proc = self.procresult.clone();
        let params = self.params.clone();

        Box::new(move |result: &ProcResult| {
            let ui_handle = ui_handle.clone();

            let maxhist = *result.histogram.1.iter().max().unwrap() as f64;
            let maxhist = f64::powf(2.0, f64::log2(maxhist).ceil());

            let histyrange = PlotRange {
                min: 0.0_f32,
                max: maxhist as f32,
            };
            let histxrange = PlotRange {
                min: *result.histogram.0.first().unwrap() as f32,
                max: *result.histogram.0.last().unwrap() as f32,
            };

            // Save the result
            *proc.write().unwrap() = result.clone();

            // Clone pointer for moving into closure
            let proc = proc.clone();
            let params = params.clone();

            // GUI is single threaded, so we must populate the image in the GUI thread
            let _ = ui_handle.upgrade_in_event_loop(move |ui| {
                let result = proc.read().unwrap();
                let global = ui.global::<Shared>();

                let histdata = slint::VecModel::from_slice(&[PlotData {
                    points: {
                        let s = slint::VecModel::default();
                        s.push(PlotPoint {
                            x: histxrange.min,
                            y: 0.0,
                        });
                        s.push(PlotPoint {
                            x: histxrange.max,
                            y: 0.0,
                        });
                        s.extend_from_slice(
                            &result
                                .histogram
                                .0
                                .iter()
                                .zip(result.histogram.1.iter())
                                .rev()
                                .map(|(x, y)| PlotPoint {
                                    x: *x as f32,
                                    y: *y as f32,
                                })
                                .collect::<Vec<PlotPoint>>(),
                        );

                        s.push(PlotPoint {
                            x: histxrange.min,
                            y: 0.0,
                        });
                        slint::ModelRc::new(s)
                    },
                    r#type: PlotType::Fill,
                    color: slint::Color::from_argb_u8(255, 255, 0, 0).darker(0.5),
                    fillcolor: slint::Color::from_argb_u8(255, 255, 0, 0).darker(1.4),
                    label: slint::SharedString::from("Histogram"),
                    linewidth: 1.0_f32,
                    markersize: 1.0_f32,
                    markertype: MarkerType::None,
                }]);

                let histxdelta = (histxrange.max - histxrange.min) / 8.0;
                let histydelta = (histyrange.max - histyrange.min) / 8.0;
                global.set_histxticks(slint::VecModel::from_slice(
                    &(0..=8)
                        .map(|i| histxrange.min + i as f32 * histxdelta)
                        .collect::<Vec<f32>>(),
                ));
                global.set_histyticks(slint::VecModel::from_slice(
                    &(0..=8)
                        .map(|i| histyrange.min + i as f32 * histydelta)
                        .collect::<Vec<f32>>(),
                ));

                // Setup the histogram
                global.set_histxrange(histxrange);
                global.set_histyrange(histyrange);
                global.set_histdata(histdata);

                // Set the frame rate
                global.set_framerate(slint::SharedString::from(format!(
                    "{:.2} fps",
                    result.framerate
                )));

                // Create the image
                ui.set_camframe_height(result.rawframe.frame.height() as i32);
                ui.set_camframe_width(result.rawframe.frame.width() as i32);

                {
                    let p = params.read().unwrap();
                    match p.fcscaletype {
                        FCScaleType::Auto => {
                            global.set_fcrange((result.fcrange.1, result.fcrange.0));
                        }
                        FCScaleType::Manual => {
                            global.set_fcrange((p.scale_range.1, p.scale_range.0));
                        }
                        FCScaleType::Max => {
                            global.set_fcrange((
                                (1_u32 << result.rawframe.bit_depth.unwrap_or(12)) as i32 - 1,
                                0,
                            ));
                        }
                    }
                }

                // Get the pixel value at the current mouse position
                let mut xpix = ui.get_xpix() as usize;
                let mut ypix = ui.get_ypix() as usize;
                xpix = xpix.clamp(0, result.rawframe.frame.width().saturating_sub(1));
                ypix = ypix.clamp(0, result.rawframe.frame.height().saturating_sub(1));
                ui.set_valatpix(match result.rawframe.frame {
                    FrameType::U8(ref f) => *f.at(xpix, ypix).unwrap_or(&0) as i32,
                    FrameType::U16(ref f) => *f.at(xpix, ypix).unwrap_or(&0) as i32,
                    FrameType::U32(ref f) => *f.at(xpix, ypix).unwrap_or(&0) as i32,
                    FrameType::I8(ref f) => *f.at(xpix, ypix).unwrap_or(&0) as i32,
                    FrameType::I16(ref f) => *f.at(xpix, ypix).unwrap_or(&0) as i32,
                    FrameType::I32(ref f) => *f.at(xpix, ypix).unwrap_or(&0),
                    _ => 0,
                });

                ui.set_meantext(slint::SharedString::from(format!(
                    "{:.2}",
                    result.mean.unwrap_or(0.0)
                )));
                ui.set_vartext(slint::SharedString::from(format!(
                    "{:.2}",
                    result.var.unwrap_or(0.0).sqrt()
                )));
                ui.window().request_redraw();
            });
        })
    }

    fn update_colorbar(ui: &AppWindow) {
        let cmap = camera::colormap::from_string(ui.global::<Shared>().get_colormap().as_str())
            .unwrap_or(camera::colormap::grayscale());
        let gamma = ui.global::<Shared>().get_gamma();

        // We have to reverse the order of the colormap to match the GUI's expectations
        let mut rcmap = cmap.iter().rev().cloned().collect::<Vec<_>>();

        // Gamma correction, if needed
        if f32::abs(gamma - 1.0) > 0.01 {
            rcmap = (0..256)
                .map(|i| {
                    let fi = i as f32 / 255.0;
                    let idx = (fi.powf(gamma) * 255.0).round() as usize;
                    rcmap[idx]
                })
                .collect::<Vec<_>>();
        }

        let cmap_image =
            Image::from_rgba8_premultiplied(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                unsafe {
                    std::slice::from_raw_parts(
                        rcmap.as_ptr() as *const u8,
                        rcmap.len() * std::mem::size_of::<Rgba8Pixel>(),
                    )
                },
                1,
                256,
            ));

        ui.set_colormap_image(cmap_image);
    }

    pub fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let ui = AppWindow::new()?;
        let params = Arc::new(RwLock::new(GuiParams::default()));

        Self::update_colorbar(&ui.as_weak().unwrap());
        params.write().unwrap().colorscale =
            ui.global::<Shared>().get_colormap().as_str().to_string();

        ui.global::<Shared>().on_mouseover_string(
            |x: i32, y: i32, val: i32| -> slint::SharedString {
                let s: char = if val < 0 { '-' } else { ' ' };
                slint::SharedString::from(format!("({:4}, {:4}) = {}{:5}", x, y, s, val).as_str())
            },
        );

        ui.global::<PlotGlobal>().on_drawline(
            |points: slint::ModelRc<PlotPoint>,
             xrange: PlotRange,
             yrange: PlotRange,
             aspect: f32|
             -> slint::SharedString {
                let xscale = 100.0 / (xrange.max - xrange.min);
                let yscale = 100.0 / (yrange.max - yrange.min) * aspect;
                let mut svg = String::new();

                // Remap to a reversible vector, then reverse
                let p: Vec<PlotPoint> = points.iter().collect();

                if p.is_empty() {
                    return slint::SharedString::from(" Z");
                }

                let m = p.first().unwrap();
                svg.push_str(
                    format!(
                        "M {} {}",
                        ((m.x - xrange.min) * xscale).clamp(0.0, 100.0),
                        (100.0 * aspect - ((m.y - yrange.min) * yscale)).clamp(0.0, 100.0 * aspect)
                    )
                    .as_str(),
                );

                p.iter().skip(1).for_each(|p| {
                    let x = ((p.x - xrange.min) * xscale).clamp(0.0, 100.0);
                    let y =
                        (100.0 * aspect - (p.y - yrange.min) * yscale).clamp(0.0, 100.0 * aspect);
                    svg.push_str(format!(" L {} {}", x, y).as_str());
                });
                svg += " Z";
                slint::SharedString::from(svg)
            },
        );

        ui.global::<PlotGlobal>().on_drawfill(
            |points: slint::ModelRc<PlotPoint>,
             xrange: PlotRange,
             yrange: PlotRange,
             aspect: f32|
             -> slint::SharedString {
                let xscale = 100.0 / (xrange.max - xrange.min);
                let yscale = 100.0 / (yrange.max - yrange.min) * aspect;
                let mut svg = String::new();

                // Remap to a reversible vector, then reverse
                let p: Vec<PlotPoint> = points.iter().collect();
                let p: Vec<PlotPoint> = p.into_iter().rev().collect();

                if p.is_empty() {
                    return slint::SharedString::from("");
                }

                let m = p.first().unwrap();
                svg.push_str(
                    format!(
                        "M {} {}",
                        ((m.x - xrange.min) * xscale).clamp(0.0, 100.0),
                        100.0 * aspect,
                    )
                    .as_str(),
                );

                let m = p.last().unwrap();
                svg.push_str(
                    format!(
                        " L {} {}",
                        ((m.x - xrange.min) * xscale).clamp(0.0, 100.0),
                        100.0 * aspect,
                    )
                    .as_str(),
                );

                p.iter().for_each(|p| {
                    let x = ((p.x - xrange.min) * xscale).clamp(0.0, 100.0);
                    let y =
                        (100.0 * aspect - (p.y - yrange.min) * yscale).clamp(0.0, 100.0 * aspect);
                    svg.push_str(format!("L {} {}", x, y).as_str());
                });

                let m = p.first().unwrap();
                svg.push_str(
                    format!(
                        "L {} {}",
                        ((m.x - xrange.min) * xscale).clamp(0.0, 100.0),
                        100.0 * aspect,
                    )
                    .as_str(),
                );
                svg += " Z";

                slint::SharedString::from(svg)
            },
        );

        ui.set_camframe_width(512);
        ui.set_camframe_height(512);
        ui.global::<Shared>().on_view_changed({
            let ui_handle = ui.as_weak();
            let params = params.clone();
            move || {
                let ui = ui_handle.unwrap();
                let globals = ui.global::<Shared>();
                Self::update_colorbar(&ui);
                let mut p = params.write().unwrap();
                p.colorscale = String::from(globals.get_colormap().as_str());
                p.gamma = globals.get_gamma() as f64;
                p.fcscaletype = match globals.get_autoscale() {
                    true => FCScaleType::Auto,
                    false => match globals.get_manualfc() {
                        true => FCScaleType::Manual,
                        false => FCScaleType::Max,
                    },
                };
                p.scale_range = (globals.get_manualfc_min(), globals.get_manualfc_max());
                ui.window().request_redraw();
            }
        });

        let procresult = Arc::new(RwLock::new(ProcResult::default()));
        ui.global::<Shared>().on_displayimage({
            let proc = procresult.clone();
            let params = params.clone();

            let mut resizer = fast_image_resize::Resizer::new();
            let mut resized =
                fast_image_resize::images::Image::new(512, 512, fast_image_resize::PixelType::U8x4);
            let mut unresized =
                fast_image_resize::images::Image::new(512, 512, fast_image_resize::PixelType::U8x4);

            move |width: f32, height: f32| {
                let width = width as usize;
                let height = height as usize;

                let res = proc.read().unwrap();
                let rawframe = &proc.read().unwrap().rawframe;

                // Get the colormap
                let cmap =
                    camera::colormap::from_string(params.read().unwrap().colorscale.as_str())
                        .unwrap_or(camera::colormap::grayscale());
                let gamma = params.read().unwrap().gamma;
                let maxcolor = 255_i64;

                let (minscale, maxscale) = match params.read().unwrap().fcscaletype {
                    FCScaleType::Auto => (res.fcrange.0 as u16, res.fcrange.1 as u16),
                    FCScaleType::Manual => (
                        params.read().unwrap().scale_range.0 as u16,
                        params.read().unwrap().scale_range.1 as u16,
                    ),
                    FCScaleType::Max => (
                        0,
                        ((1_u32 << rawframe.bit_depth.unwrap_or(12) as u32) - 1) as u16,
                    ),
                };
                let range = (maxscale - minscale).max(1);

                if (unresized.width(), unresized.height())
                    != (
                        rawframe.frame.width() as u32,
                        rawframe.frame.height() as u32,
                    )
                {
                    unresized = fast_image_resize::images::Image::new(
                        rawframe.frame.width() as u32,
                        rawframe.frame.height() as u32,
                        fast_image_resize::PixelType::U8x4,
                    );
                }

                let cbuf = unresized.buffer_mut();
                match rawframe.frame {
                    FrameType::U8(ref f) => {
                        f.data().iter().enumerate().for_each(|(i, x)| {
                            let idx = match (gamma - 1.0).abs() < 0.02 {
                                true => ((*x as i64 - minscale as i64) * maxcolor / range as i64)
                                    .clamp(0, 255) as usize,
                                false => (((*x as f32 - minscale as f32) / range as f32)
                                    .powf(1.0 / gamma as f32)
                                    * maxcolor as f32)
                                    .clamp(0.0, 255.0)
                                    as usize,
                            };
                            cbuf[i * 4] = cmap[idx].r;
                            cbuf[i * 4 + 1] = cmap[idx].g;
                            cbuf[i * 4 + 2] = cmap[idx].b;
                            cbuf[i * 4 + 3] = cmap[idx].a;
                        });
                    }
                    FrameType::U16(ref f) => f.data().iter().enumerate().for_each(|(i, x)| {
                        let idx = match (gamma - 1.0).abs() < 0.02 {
                            true => ((*x as i64 - minscale as i64) * maxcolor / range as i64)
                                .clamp(0, 255) as usize,
                            false => (((*x as f32 - minscale as f32) / range as f32)
                                .powf(1.0 / gamma as f32)
                                * maxcolor as f32)
                                .clamp(0.0, 255.0) as usize,
                        };
                        cbuf[i * 4] = cmap[idx].r;
                        cbuf[i * 4 + 1] = cmap[idx].g;
                        cbuf[i * 4 + 2] = cmap[idx].b;
                        cbuf[i * 4 + 3] = cmap[idx].a;
                    }),

                    _ => {}
                };

                // See if we need to re-allocate destination
                if (resized.width(), resized.height()) != (width as u32, height as u32) {
                    resized = fast_image_resize::images::Image::new(
                        width as u32,
                        height as u32,
                        fast_image_resize::PixelType::U8x4,
                    );
                }

                resizer.resize(&unresized, &mut resized, None).unwrap();

                let cmap_image = Image::from_rgba8_premultiplied(
                    SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                        resized.buffer(),
                        resized.width(),
                        resized.height(),
                    ),
                );
                cmap_image
            }
        });

        let gui = Self {
            ui: Rc::new(RefCell::new(ui.clone_strong())),
            params,
            procresult,
            camera: None,
            proc: ImageProcessor::new(),
        };

        Ok(gui)
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.ui.borrow_mut().run()?;
        if let Some(cam) = &mut self.camera {
            println!("stopping camera");
            cam.stop()?;
            println!("disconnecting camera");
            // sleep for 10 milliseconds
            std::thread::sleep(std::time::Duration::from_millis(10));
            cam.disconnect()?;
        }
        Ok(())
    }
}
