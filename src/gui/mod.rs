use crate::cameraframe::CameraFrame;
use crate::cameraframe::FrameData;
use crate::cameraframe::MonoPixel;
use crate::cameraframe::RGBAPixel;
use crate::imgproc::ProcResult;
use std::error::Error;

use slint::Image;
use slint::Model;
use slint::Rgba8Pixel;
use slint::SharedPixelBuffer;

slint::include_modules!();

use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct GuiParams {
    pub gamma: f64,
    pub auto_scale: bool,
    pub scale_range: (i32, i32),
    pub colorscale: String,
}
impl Default for GuiParams {
    fn default() -> Self {
        Self {
            gamma: 1.0,
            auto_scale: true,
            scale_range: (0, 65535),
            colorscale: "parula".to_string(),
        }
    }
}

pub struct Gui {
    pub ui: AppWindow,
    pub params: Arc<RwLock<GuiParams>>,
}

impl Gui {
    pub fn get_params(&self) -> Arc<RwLock<GuiParams>> {
        self.params.clone()
    }

    pub fn get_params_callback(&self) -> Box<dyn Fn() -> GuiParams + Send + 'static> {
        let params = self.params.clone();
        Box::new(move || params.read().unwrap().clone())
    }

    pub fn processed_callback<T>(&self) -> Box<dyn Fn(ProcResult<T>) + Send + 'static>
    where
        T: MonoPixel,
    {
        let ui_handle: slint::Weak<AppWindow> = self.ui.as_weak().clone();
        let params = self.get_params();
        Box::new(move |result: ProcResult<T>| {
            let ui_handle = ui_handle.clone();
            let params = params.clone();
            // GUI is single threaded, so we must populate the image in the GUI thread
            let _ = slint::invoke_from_event_loop(move || {
                let ui = ui_handle.unwrap();
                ui.set_camframe_height(result.displayimage.height as i32);
                ui.set_camframe_width(result.displayimage.width as i32);
                ui.set_camframe(Image::from_rgba8_premultiplied(SharedPixelBuffer::<
                    Rgba8Pixel,
                >::clone_from_slice(
                    unsafe {
                        std::slice::from_raw_parts(
                            result.displayimage.data.as_ptr() as *const u8,
                            result.displayimage.data.len() * std::mem::size_of::<Rgba8Pixel>(),
                        )
                    },
                    result.displayimage.width,
                    result.displayimage.height,
                )));
            });
        })
    }

    fn update_colorbar(ui: &AppWindow) {
        let cmap = crate::colormap::from_string(ui.get_colormap().as_str())
            .unwrap_or(crate::colormap::grayscale());
        let gamma = ui.get_gamma();

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

    pub fn new() -> Result<Self, Box<dyn Error>> {
        let ui = AppWindow::new()?;
        let params = Arc::new(RwLock::new(GuiParams::default()));

        Self::update_colorbar(&ui.as_weak().unwrap());
        params.write().unwrap().colorscale = ui.get_colormap().as_str().to_string();

        // take a list of points and x,y ranges and return an SVG path as a string
        // This is very inelegant, but it works
        ui.global::<Shared>().on_linetosvg(
            |p: slint::ModelRc<(f32, f32)>,
             xrange: (f32, f32),
             yrange: (f32, f32)|
             -> slint::SharedString {
                let xscale = 100.0 / (xrange.1 - xrange.0);
                let yscale = 100.0 / (yrange.1 - yrange.0);
                let mut svg = String::new();
                let m = p.iter().next().unwrap();
                svg.push_str("M 0 0 ");
                svg.push_str(
                    format!(
                        "M {} {}",
                        (100.0 - (m.0 - xrange.0) * xscale).clamp(0.0, 100.0),
                        ((m.1 - yrange.0) * yscale).clamp(0.0, 100.0)
                    )
                    .as_str(),
                );

                p.iter().for_each(|(x, y)| {
                    let x = (100.0 - (x - xrange.0) * xscale).clamp(0.0, 100.0);
                    let y = ((y - yrange.0) * yscale).clamp(0.0, 100.0);
                    svg.push_str(format!(" L {} {}", x, y).as_str());
                });

                svg.push_str(" M 100 100 Z");
                slint::SharedString::from(svg)
            },
        );

        ui.set_camframe_width(512);
        ui.set_camframe_height(512);
        ui.on_colormap_changed({
            let ui_handle = ui.as_weak();
            let params = params.clone();
            move || {
                let ui = ui_handle.unwrap();
                Self::update_colorbar(&ui);
                params.write().unwrap().colorscale = ui.get_colormap().as_str().to_string();
            }
        });
        ui.on_gamma_changed({
            let params = params.clone();
            let ui_handle = ui.as_weak();
            move |gamma| {
                // Update external gui parameter table
                let mut params = params.write().unwrap();
                params.gamma = gamma as f64;
                // internally, update the colormap on the colorbar
                Self::update_colorbar(&ui_handle.unwrap());
            }
        });

        let gui = Self { ui, params };
        Ok(gui)
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.ui.run()?;
        Ok(())
    }
}
