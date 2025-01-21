use crate::CameraFrame;

use super::procresult::ProcResult;
use crate::cameraframe::MonoPixel;
use std::sync::{Arc, Mutex, RwLock};

use crate::gui::GuiParams;

pub struct ImageProcessor<T>
where
    T: MonoPixel,
{
    params: Option<Arc<RwLock<GuiParams>>>,
    sink: Option<Box<dyn Fn(ProcResult<T>) + 'static + Send>>,
    lastresult: Option<ProcResult<T>>,
}

impl<T> ImageProcessor<T>
where
    T: MonoPixel,
{
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(ImageProcessor::<T> {
            params: None,
            sink: None,
            lastresult: None,
        }))
    }

    pub fn set_params(&mut self, params: Arc<RwLock<GuiParams>>) {
        self.params = Some(params);
    }

    pub fn set_sink(&mut self, sink: impl Fn(ProcResult<T>) + 'static + Send) {
        self.sink = Some(Box::new(sink));
    }

    fn compute_histogram(frame: &CameraFrame<T>) -> (Vec<i32>, Vec<i32>) {
        let min = frame.data.minval();
        let max = frame.data.maxval();
        let histmin = f64::powf(2.0, f64::log2(min.to_f64().unwrap()).floor());
        let histmax = f64::powf(2.0, f64::log2(max.to_f64().unwrap()).ceil());
        let nbins = 256;
        let histdelta: f64 = (histmax - histmin) / nbins as f64;
        let bins = (0..nbins)
            .map(|i| (histmin + i as f64 * histdelta) as i32)
            .collect::<Vec<i32>>();
        let mut hist = vec![0; nbins];

        frame.data.data.iter().for_each(|&x| {
            let bin = ((x.to_f64().unwrap() - histmin) / histdelta).floor() as usize;
            hist[bin] += 1;
        });
        (bins, hist)
    }

    pub fn process_frame(&mut self, frame: CameraFrame<T>) {
        // Parameters for the GUI
        let params = match &self.params {
            Some(f) => f.read().unwrap().clone(),
            None => GuiParams::default(),
        };

        let cmap = crate::colormap::from_string(params.colorscale.as_str())
            .unwrap_or(crate::colormap::grayscale());

        let (minscale, maxscale) = match params.auto_scale {
            true => (frame.data.minval(), frame.data.maxval()),
            false => (
                T::from(params.scale_range.0).unwrap(),
                T::from(params.scale_range.1).unwrap(),
            ),
        };

        let rgbaframe = frame.data.to_rgba(minscale, maxscale, params.gamma, cmap);

        let histogram = Self::compute_histogram(&frame);

        let result = ProcResult {
            rawframe: frame,
            displayimage: rgbaframe,
            histogram,
            fcrange: (minscale.to_i32().unwrap(), maxscale.to_i32().unwrap()),
        };
        if let Some(cb) = &self.sink {
            self.lastresult = Some(result.clone());
            cb(result);
        } else {
            self.lastresult = Some(result);
        }
    }
}
