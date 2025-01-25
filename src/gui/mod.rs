use crate::imgproc::ProcResult;
use std::error::Error;

use slint::Image;
use slint::Model;
use slint::Rgba8Pixel;
use slint::SharedPixelBuffer;

slint::include_modules!();

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

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
            colorscale: "parula".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Gui {
    pub ui: Rc<RefCell<AppWindow>>,
    pub params: Arc<RwLock<GuiParams>>,
    pub proc: Arc<RwLock<ProcResult<u16>>>,
}

impl Gui {
    pub fn get_params(&self) -> Arc<RwLock<GuiParams>> {
        self.params.clone()
    }

    pub fn on_processed(&self) -> Box<dyn Fn(ProcResult<u16>) + Send + 'static> {
        let ui_handle: slint::Weak<AppWindow> = self.ui.borrow().as_weak();
        let proc = self.proc.clone();

        Box::new(move |result: ProcResult<u16>| {
            let ui_handle = ui_handle.clone();
            let histxrange = (
                *result.histogram.0.last().unwrap() as f32,
                *result.histogram.0.first().unwrap() as f32,
            );

            let maxhist = *result.histogram.1.iter().max().unwrap() as f64;
            let maxhist = f64::powf(2.0, f64::log2(maxhist).ceil());

            let histyrange = (maxhist as f32, 0.0_f32);

            // Save the result
            *proc.write().unwrap() = result;
            // Clone pointer for moving into closure
            let proc = proc.clone();

            // GUI is single threaded, so we must populate the image in the GUI thread
            let _ = ui_handle.upgrade_in_event_loop(move |ui| {
                let result = proc.read().unwrap();
                let global = ui.global::<Shared>();
                // Create the histogram points

                let histpoints = slint::VecModel::from_slice(
                    &result
                        .histogram
                        .0
                        .iter()
                        .zip(result.histogram.1.iter())
                        .map(|(x, y)| (*x as f32, *y as f32))
                        .collect::<Vec<(f32, f32)>>(),
                );
                let histline = slint::VecModel::from_slice(&[(
                    slint::Color::from_argb_u8(255, 255, 0, 0),
                    1.5_f32,
                    histpoints,
                )]);
                // Setup the histogram
                global.set_histxrange(histxrange);
                global.set_histyrange(histyrange);
                global.set_histdata(histline);

                // Create the image
                ui.set_camframe_height(result.rawframe.data.height as i32);
                ui.set_camframe_width(result.rawframe.data.width as i32);

                global.set_fcrange((result.fcrange.1, result.fcrange.0));

                let xpix = ui.get_xpix() as u32;
                let ypix = ui.get_ypix() as u32;
                ui.set_valatpix(result.rawframe.data.at(xpix, ypix) as i32);

                let (mean, var) = result.rawframe.data.mean_and_var();
                ui.set_meantext(slint::SharedString::from(format!("{:.2}", mean)));
                ui.set_vartext(slint::SharedString::from(format!("{:.2}", var.sqrt())));
                ui.window().request_redraw();
            });
        })
    }

    fn update_colorbar(ui: &AppWindow) {
        let cmap = crate::colormap::from_string(ui.global::<Shared>().get_colormap().as_str())
            .unwrap_or(crate::colormap::grayscale());
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

        ui.global::<Shared>()
            .on_triangle_string(|a: f32| -> slint::SharedString {
                let v1 = [0.0, -75.0];
                let v2 = [-75.0, 75.0];
                let v3 = [75.0, 75.0];

                let angle = (90.0 + a * 90.0).to_radians();
                let v1 = [
                    v1[0] * angle.cos() - v1[1] * angle.sin(),
                    v1[0] * angle.sin() + v1[1] * angle.cos(),
                ];
                let v2 = [
                    v2[0] * angle.cos() - v2[1] * angle.sin(),
                    v2[0] * angle.sin() + v2[1] * angle.cos(),
                ];
                let v3 = [
                    v3[0] * angle.cos() - v3[1] * angle.sin(),
                    v3[0] * angle.sin() + v3[1] * angle.cos(),
                ];
                let v1 = [v1[0] + 75.0, v1[1] + 75.0];
                let v2 = [v2[0] + 75.0, v2[1] + 75.0];
                let v3 = [v3[0] + 75.0, v3[1] + 75.0];
                slint::SharedString::from(
                    format!(
                        "M {} {} L {} {} L {} {} L {} {} Z",
                        v1[0], v1[1], v2[0], v2[1], v3[0], v3[1], v1[0], v1[1]
                    )
                    .as_str(),
                )
            });

        ui.global::<Shared>().on_mouseover_string(
            |x: i32, y: i32, val: i32| -> slint::SharedString {
                let s: char = if val < 0 { '-' } else { ' ' };
                slint::SharedString::from(format!("({:4}, {:4}) = {}{:5}", x, y, s, val).as_str())
            },
        );

        // take a list of points and x,y ranges and return an SVG path as a string
        // This is very inelegant, but it works
        ui.global::<Shared>().on_linetosvg(
            |p: slint::ModelRc<(f32, f32)>,
             xrange: (f32, f32),
             yrange: (f32, f32),
             aspect: f32|
             -> slint::SharedString {
                let xscale = 100.0 / (xrange.1 - xrange.0);
                let yscale = 100.0 / (yrange.1 - yrange.0) * aspect;
                let mut svg = String::new();
                let m = p.iter().next().unwrap();
                svg.push_str(
                    format!(
                        "M {} {}",
                        (100.0 - (m.0 - xrange.0) * xscale).clamp(0.0, 100.0),
                        ((m.1 - yrange.0) * yscale).clamp(0.0, 100.0 * aspect)
                    )
                    .as_str(),
                );

                p.iter().skip(1).for_each(|(x, y)| {
                    let x = (100.0 - (x - xrange.0) * xscale).clamp(0.0, 100.0);
                    let y = ((y - yrange.0) * yscale).clamp(0.0, 100.0 * aspect);
                    svg.push_str(format!(" L {} {}", x, y).as_str());
                });

                svg.push_str(" M 0 0 Z");
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
                println!("requesting redraw");
                let sz = ui.window().size();
                ui.window()
                    .dispatch_event(slint::platform::WindowEvent::Resized {
                        size: sz.to_logical(1.0),
                    });
            }
        });

        let proc = Arc::new(RwLock::new(ProcResult::default()));
        ui.global::<Shared>().on_displayimage({
            let proc = proc.clone();
            let params = params.clone();

            let mut resizer = fast_image_resize::Resizer::new();
            let mut resized =
                fast_image_resize::images::Image::new(512, 512, fast_image_resize::PixelType::U8x4);
            let mut unresized =
                fast_image_resize::images::Image::new(512, 512, fast_image_resize::PixelType::U8x4);

            move |width: f32, height: f32| {
                let width = width as usize;
                let height = height as usize;

                let rawframe = &proc.read().unwrap().rawframe;
                let raw = &proc.read().unwrap().rawframe.data;

                // Get the colormap
                let cmap = crate::colormap::from_string(params.read().unwrap().colorscale.as_str())
                    .unwrap_or(crate::colormap::grayscale());
                let gamma = params.read().unwrap().gamma;
                let maxcolor = 255_i64;
                let (minscale, maxscale) = match params.read().unwrap().fcscaletype {
                    FCScaleType::Auto => (raw.minval(), raw.maxval()),
                    FCScaleType::Manual => (
                        params.read().unwrap().scale_range.0 as u16,
                        params.read().unwrap().scale_range.1 as u16,
                    ),
                    FCScaleType::Max => (0, (1_u32 << rawframe.bit_depth as u32) as u16),
                };
                let range = maxscale - minscale;

                if (unresized.width(), unresized.height()) != (raw.width, raw.height) {
                    unresized = fast_image_resize::images::Image::new(
                        raw.width,
                        raw.height,
                        fast_image_resize::PixelType::U8x4,
                    );
                }

                let cbuf = unresized.buffer_mut();

                // Create the RGBA image
                raw.data.iter().enumerate().for_each(|(i, x)| {
                    let idx = match (gamma - 1.0).abs() < 0.02 {
                        true => (((*x as i64 - minscale as i64) * maxcolor / range as i64)
                            .clamp(0, 255) as usize)
                            .min(255),
                        false => (((*x as f32 - minscale as f32) / range as f32)
                            .powf(1.0 / gamma as f32)
                            * maxcolor as f32)
                            .clamp(0.0, 255.0) as usize,
                    };
                    let color = cmap[idx];
                    cbuf[i * 4] = color.r;
                    cbuf[i * 4 + 1] = color.g;
                    cbuf[i * 4 + 2] = color.b;
                    cbuf[i * 4 + 3] = color.a;
                });

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
                        width as u32,
                        height as u32,
                    ),
                );
                cmap_image
            }
        });

        let gui = Self {
            ui: Rc::new(RefCell::new(ui)),
            params,
            proc,
        };

        Ok(gui)
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.ui.borrow_mut().run()?;
        Ok(())
    }
}
