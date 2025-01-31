use camera::MonoCameraFrame;
use camera::MonoPixel;

///
/// Output of image processing chain
///
/// # Contains
/// * raw image,
/// * Image with contigious memory to be displayed in color format
/// * False color range ued in the display
/// * Histogram of the image
///
#[derive(Clone)]
pub struct ProcResult<T>
where
    T: MonoPixel,
{
    pub rawframe: MonoCameraFrame<T>,
    pub histogram: (Vec<i32>, Vec<i32>),
    pub fcrange: (i32, i32),
}

impl<T> Default for ProcResult<T>
where
    T: MonoPixel,
{
    fn default() -> Self {
        ProcResult {
            rawframe: MonoCameraFrame::<T>::default(),
            histogram: (vec![], vec![]),
            fcrange: (0, 4096),
        }
    }
}
