use super::FrameData;
use super::MonoPixel;
use super::Pixel;

#[derive(Clone)]
pub struct CameraFrame<T>
where
    T: Pixel,
{
    pub exposure: f64,
    pub center_of_integration: chrono::DateTime<chrono::Utc>,
    pub bit_depth: u8,
    pub data: FrameData<T>,
}

//pub type CameraFrameU16 = CameraFrame<u16>;
//pub type CameraFrameU32 = CameraFrame<u32>;
//pub type CameraFrameI32 = CameraFrame<i32>;

impl<T> CameraFrame<T>
where
    T: Pixel,
{
    /// Create a new camera frame with given exposure, center of integration, bit depth, and raw data.
    ///
    /// # Arguments
    /// * `exposure` - The exposure time of the frame in seconds.
    /// * `center_of_integration` - The time at the center of the integration period.
    /// * `bit_depth` - The bit depth of the frame.
    /// * `raw` - The raw data of the frame.
    ///
    /// # Returns
    /// A new camera frame with the given exposure, center of integration, bit depth, and raw data.
    ///
    pub fn create(
        exposure: f64,
        center_of_integration: chrono::DateTime<chrono::Utc>,
        bit_depth: u8,
        raw: FrameData<T>,
    ) -> CameraFrame<T> {
        CameraFrame {
            exposure,
            center_of_integration,
            bit_depth,
            data: raw,
        }
    }
}

impl<T> Default for CameraFrame<T>
where
    T: Pixel,
{
    fn default() -> Self {
        CameraFrame {
            exposure: 0.0,
            center_of_integration: chrono::Utc::now(),
            bit_depth: 12,
            data: FrameData::<T>::default(),
        }
    }
}
