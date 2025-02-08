use super::procresult::ProcResult;

use camera::CameraFrame;
use camera::PixelType;
use rgb::bytemuck;

use std::sync::{Arc, Mutex, RwLock};

use crate::gui::GuiParams;

type ResultCallback = dyn Fn(&ProcResult) + 'static + Send;

pub struct ImageProcessor {
    params: Option<Arc<RwLock<GuiParams>>>,
    sink: Option<Box<ResultCallback>>,
    lastresult: Option<ProcResult>,
}

impl ImageProcessor {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(ImageProcessor {
            params: None,
            sink: None,
            lastresult: None,
        }))
    }

    pub fn set_params(&mut self, params: Arc<RwLock<GuiParams>>) {
        self.params = Some(params);
    }

    pub fn set_sink(&mut self, sink: impl Fn(&ProcResult) + 'static + Send) {
        self.sink = Some(Box::new(sink));
    }

    fn compute_mean_and_var<T>(pixels: &[T]) -> (f64, f64)
    where
        T: num_traits::PrimInt,
    {
        let (sum, smusq) = pixels.iter().fold((0_i64, 0_i64), |(sum, smusq), &x| {
            let x = x.to_i64().unwrap();
            (sum + x, smusq + x * x)
        });

        let n = (pixels.len() as f64).max(2.0);
        let mean = sum as f64 / n;
        let var = (smusq as f64 - n * mean * mean) / (n - 1.0);
        (mean, var)
    }

    fn compute_histogram<T>(pixels: &[T]) -> (i32, i32, Vec<i32>, Vec<i32>)
    where
        T: num_traits::PrimInt,
    {
        let mut min = T::max_value();
        let mut max = T::min_value();

        pixels.iter().for_each(|&x| {
            min = min.min(x);
            max = max.max(x);
        });

        let histmin = f64::powf(2.0, f64::log2(min.to_f64().unwrap()).floor());
        let histmin = f64::min(histmin, 0.0);
        let histmax = f64::powf(2.0, f64::log2(max.to_f64().unwrap()).ceil());
        let nbins = 256;
        let histdelta = ((histmax - histmin) / nbins as f64).ceil() as i32;

        let histmin = histmin as i32;
        let bins = (0..(nbins + 1))
            .map(|i| histmin + i * histdelta)
            .collect::<Vec<i32>>();
        let mut hist = vec![0; (nbins + 1) as usize];
        pixels.iter().for_each(|&x| {
            let bin = ((x.to_i32().unwrap() - histmin) / histdelta) as usize;
            hist[bin] += 1;
        });

        (min.to_i32().unwrap(), max.to_i32().unwrap(), bins, hist)
    }

    fn process_mono(&mut self, frame: CameraFrame) {
        let (minval, maxval, bins, hist) = match frame.pixeltype {
            PixelType::Gray8 => Self::compute_histogram(&frame.data),
            PixelType::Gray16 => {
                Self::compute_histogram(bytemuck::cast_slice::<u8, u16>(&frame.data))
            }
            _ => (0, 4096, vec![], vec![]),
        };
        let (mean, var) = match frame.pixeltype {
            PixelType::Gray8 => Self::compute_mean_and_var(&frame.data),
            PixelType::Gray16 => {
                Self::compute_mean_and_var(bytemuck::cast_slice::<u8, u16>(&frame.data))
            }
            _ => (0.0, 0.0),
        };

        // Create the result
        let res = ProcResult {
            rawframe: frame,
            histogram: (bins, hist),
            fcrange: (minval, maxval),
            mean: Some(mean),
            var: Some(var),
        };

        // Store the result
        self.lastresult = Some(res);

        // Process the result
        if let Some(sink) = &self.sink {
            sink(self.lastresult.as_ref().unwrap());
        }
    }

    ///
    /// Process a raw frame to produce a result.
    ///
    /// Then run the "sink" function on that result when complete
    ///
    pub fn process_frame(&mut self, frame: CameraFrame) {
        if frame.pixeltype.is_mono() {
            self.process_mono(frame)
        }
    }
}
