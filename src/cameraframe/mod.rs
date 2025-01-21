mod cameraframe_def;
mod framedata;
mod mono_cast;
mod mono_ops;
mod mono_stats;
mod pixel;
mod to_file;

/// Monochromatic pixel type
pub use pixel::MonoPixel;
/// Pixel types
/// Base type
pub use pixel::Pixel;
/// RGBA pixel type
pub use pixel::RGBAPixel;
/// RGB pixel type
pub use pixel::RGBPixel;

pub use cameraframe_def::CameraFrame;
//pub use cameraframe::CameraFrameI32;
//pub use cameraframe::CameraFrameU16;
//pub use cameraframe::CameraFrameU32;
pub use framedata::FrameData;

pub struct ROI {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}
