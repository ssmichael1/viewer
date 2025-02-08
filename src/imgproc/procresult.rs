use camera::CameraFrame;

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
pub struct ProcResult {
    pub rawframe: CameraFrame,
    pub histogram: (Vec<i32>, Vec<i32>),
    pub fcrange: (i32, i32),
    pub mean: Option<f64>,
    pub var: Option<f64>,
}

impl Default for ProcResult {
    fn default() -> Self {
        ProcResult {
            rawframe: CameraFrame::default(),
            histogram: (vec![], vec![]),
            fcrange: (0, 4096),
            mean: None,
            var: None,
        }
    }
}
