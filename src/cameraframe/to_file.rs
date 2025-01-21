use super::FrameData;
use super::MonoPixel;
use super::RGBAPixel;
use super::RGBPixel;

use std::fs::File;

impl<T> FrameData<T>
where
    T: MonoPixel,
{
    /// Save the FrameData to a PNG file.
    ///
    /// # Arguments
    /// `filename` - The name of the file to save the PNG to.
    ///
    /// # Returns
    /// An empty Result if the save was successful, or an error if the save failed.
    ///
    pub fn save_to_png(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut encoder = png::Encoder::new(File::create(filename)?, self.width, self.height);
        encoder.set_color(png::ColorType::Grayscale);
        let mut bitdepth = png::BitDepth::Eight;
        if T::max_value().to_u32().unwrap() > 255 {
            bitdepth = png::BitDepth::Sixteen;
        }
        if T::max_value().to_u32().unwrap() > 65535 {
            return Err("Pixel value too large for PNG".into());
        }
        encoder.set_depth(bitdepth);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(unsafe {
            std::slice::from_raw_parts(
                self.data
                    .iter()
                    .map(|x| x.swap_bytes())
                    .collect::<Vec<T>>()
                    .as_ptr() as *const u8,
                self.data.len() * std::mem::size_of::<T>(),
            )
        })?;
        Ok(())
    }
}

impl FrameData<RGBAPixel> {
    /// Save the FrameData to a PNG file.
    ///
    /// # Arguments
    /// `filename` - The name of the file to save the PNG to.
    ///
    /// # Returns
    /// An empty Result if the save was successful, or an error if the save failed.
    ///    
    pub fn save_to_png(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut encoder = png::Encoder::new(File::create(filename)?, self.width, self.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(unsafe {
            std::slice::from_raw_parts(
                self.data.as_ptr() as *const u8,
                self.data.len() * std::mem::size_of::<RGBAPixel>(),
            )
        })?;
        Ok(())
    }
}

impl FrameData<RGBPixel> {
    /// Save the FrameData to a PNG file.
    ///
    /// # Arguments
    /// `filename` - The name of the file to save the PNG to.
    ///
    /// # Returns
    /// An empty Result if the save was successful, or an error if the save failed.
    ///    
    pub fn save_to_png(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut encoder = png::Encoder::new(File::create(filename)?, self.width, self.height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(unsafe {
            std::slice::from_raw_parts(
                self.data.as_ptr() as *const u8,
                self.data.len() * std::mem::size_of::<RGBPixel>(),
            )
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_data() -> FrameData<u16> {
        FrameData::<u16> {
            width: 256,
            height: 256,
            data: (0..256 * 256)
                .map(|x| {
                    let mut row = (x % 256) as f64;
                    let mut col = (x / 256) as f64;
                    row -= 128.0;
                    row -= 10.0;
                    col -= 128.0;
                    col += 20.0;
                    (f64::exp(-(row * row + col * col) / 400.0) * 65535.0) as u16
                })
                .collect(),
        }
    }

    #[test]
    fn test_save_to_png() {
        let data = test_data();
        let filename = "test.png";
        let _ = std::fs::remove_file(filename);
        let data2 = &data / 256;
        let data3: FrameData<u8> = (&data2).into();
        let data4 = data3.to_rgba(0, 255, 1.0, crate::colormap::parula());
        data4.save_to_png(filename).unwrap();
        assert!(std::fs::metadata(filename).is_ok());
        //let _ = std::fs::remove_file(filename);
    }
}
