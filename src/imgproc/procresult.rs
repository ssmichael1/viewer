use crate::cameraframe::FrameData;
use crate::cameraframe::MonoPixel;
use crate::cameraframe::RGBAPixel;
use crate::CameraFrame;

#[derive(Clone)]
pub struct ProcResult<T>
where
    T: MonoPixel,
{
    pub rawframe: CameraFrame<T>,
    pub displayimage: FrameData<RGBAPixel>,
    pub histogram: (Vec<i32>, Vec<i32>),
    pub fcrange: (i32, i32),
}
