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
}

impl<T> ImageProcessor<T>
where
    T: MonoPixel,
{
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(ImageProcessor::<T> {
            params: None,
            sink: None,
        }))
    }

    pub fn set_params(&mut self, params: Arc<RwLock<GuiParams>>) {
        self.params = Some(params);
    }

    pub fn set_sink(&mut self, sink: impl Fn(ProcResult<T>) + 'static + Send) {
        self.sink = Some(Box::new(sink));
    }

    pub fn process_frame(&self, frame: CameraFrame<T>) {
        // Parameters for the GUI
        let params = match &self.params {
            Some(f) => f.read().unwrap().clone(),
            None => GuiParams::default(),
        };

        let cmap = crate::colormap::from_string(params.colorscale.as_str())
            .unwrap_or(crate::colormap::grayscale());

        let rgbaframe = frame.data.to_rgba(
            T::from(params.scale_range.0).unwrap(),
            T::from(params.scale_range.1).unwrap(),
            params.gamma,
            cmap,
        );

        let result = ProcResult {
            rawframe: frame,
            displayimage: rgbaframe,
        };
        if let Some(cb) = &self.sink {
            cb(result);
        }
    }
}
