use super::Pixel;

#[derive(Debug, Clone)]
pub struct FrameData<T>
where
    T: Pixel,
{
    pub width: u32,
    pub height: u32,
    pub data: Vec<T>,
}

impl<T> FrameData<T>
where
    T: Pixel,
{
    /// Get the value at the given x and y coordinates.
    ///
    /// # Arguments:
    /// * `x` - The x coordinate (column) of the value to get.
    /// * `y` - The y coordinate (row) of the value to get.
    ///
    /// # Returns
    /// The value at the given x and y coordinates.
    #[inline]
    pub fn at(&self, x: u32, y: u32) -> T {
        self.data[self.index(x, y)]
    }

    /// Get the value at the given x and y coordinates by reference.
    ///
    /// # Arguments:
    /// * `x` - The x coordinate (column) of the value to get.
    /// * `y` - The y coordinate (row) of the value to get.
    ///
    /// # Returns
    /// A reference to the value at the given x and y coordinates.
    #[inline]
    pub fn at_ref(&self, x: u32, y: u32) -> &T {
        &self.data[self.index(x, y)]
    }

    /// Get a subregion of the CameraFrame.
    /// The subregion is defined by the top-left and bottom-right corners.
    /// The top-left corner is inclusive, and the bottom-right corner is exclusive.
    /// The subregion is copied into a new CameraFrame.
    ///
    /// # Arguments:
    /// * `top_left_x` - The x coordinate (column) of the top-left corner of the subregion.
    /// * `top_left_y` - The y coordinate (row) of the top-left corner of the subregion.
    /// * `bottom_right_x` - The x coordinate (column) of the bottom-right corner of the subregion.
    /// * `bottom_right_y` - The y coordinate (row) of the bottom-right corner of the subregion.
    ///
    /// # Returns
    /// A new CameraFrame containing the subregion.
    ///
    /// # Panics
    /// Panics if the top-left corner is not above and to the left of the bottom-right corner.
    /// Panics if the bottom-right corner is not within the bounds of the CameraFrame.
    /// Panics if the width of the subregion is zero.
    /// Panics if the height of the subregion is zero.
    /// Panics if the width of the subregion is greater than the width of the CameraFrame.
    /// Panics if the height of the subregion is greater than the height of the CameraFrame.
    ///
    /// # Example
    /// ```
    /// use cameraframe::CameraFrame;
    /// let frame = CameraFrame::<u8>::new(3, 3);
    /// let subregion = frame.subregion(0, 0, 2, 2);
    /// assert_eq!(subregion.width, 2);
    /// assert_eq!(subregion.height, 2);
    /// ```
    ///
    pub fn subregion(
        &self,
        top_left_x: u32,
        top_left_y: u32,
        bottom_right_x: u32,
        bottom_right_y: u32,
    ) -> Self {
        assert!(top_left_x < bottom_right_x);
        assert!(top_left_y < bottom_right_y);
        assert!(bottom_right_x <= self.width);
        assert!(bottom_right_y <= self.height);

        let width = bottom_right_x - top_left_x;
        let height = bottom_right_y - top_left_y;
        let mut data = Vec::with_capacity((width * height) as usize);

        for y in top_left_y..bottom_right_y {
            for x in top_left_x..bottom_right_x {
                data.push(self.at(x, y));
            }
        }

        FrameData {
            width,
            height,
            data,
        }
    }

    /// Index into the lower-level 1D array
    ///
    /// # Arguments:
    /// * `x` - The x coordinate (column) of the value to get.
    /// * `y` - The y coordinate (row) of the value to get.
    ///
    /// # Returns
    /// The index into the lower-level 1D array.
    #[inline]
    fn index(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }
}
