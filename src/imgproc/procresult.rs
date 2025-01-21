use crate::cameraframe::FrameData;
use crate::cameraframe::MonoPixel;
use crate::cameraframe::RGBAPixel;
use crate::CameraFrame;

pub struct ProcResult<T>
where
    T: MonoPixel,
{
    pub rawframe: CameraFrame<T>,
    pub displayimage: FrameData<RGBAPixel>,
}
