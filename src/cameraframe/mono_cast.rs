//!
//! This module contains functions that enable casting of FrameData between different pixel types.
//!

use super::FrameData;
use super::MonoPixel;
use super::RGBAPixel;
use crate::colormap::ColorMap;

/// Implement the conversion from a FrameData with a different pixel type to a FrameData with the current pixel type.
/// where pixel types are monochromatic.
impl<T, T2> From<&FrameData<T2>> for FrameData<T>
where
    T: MonoPixel,
    T2: MonoPixel,
{
    fn from(other: &FrameData<T2>) -> FrameData<T> {
        FrameData::<T> {
            width: other.width,
            height: other.height,
            data: other.data.iter().map(|x| T::from(*x).unwrap()).collect(),
        }
    }
}

impl<T> FrameData<T>
where
    T: MonoPixel,
{
    /// Convert the FrameData to an RGBA FrameData.
    ///
    /// # Arguments
    /// * `minscale` - The value of the data that matches the minimum value of the color map.
    /// * `maxscale` - The value of the data that matches the maximum value of the color map.
    /// * `cmap` - The color map to use.
    ///
    /// # Returns
    /// An RGBA FrameData.
    ///
    pub fn to_rgba(
        &self,
        minscale: T,
        maxscale: T,
        gamma: f64,
        cmap: &ColorMap,
    ) -> FrameData<RGBAPixel> {
        let maxcolor = 255_i64;
        let minscale = minscale.to_i64().unwrap();
        let maxscale = maxscale.to_i64().unwrap();
        let range = maxscale - minscale;

        if f64::abs(gamma - 1.0) < 0.02 {
            FrameData::<RGBAPixel> {
                width: self.width,
                height: self.height,
                data: self
                    .data
                    .iter()
                    .map(|x| {
                        let idx = (((*x).to_i64().unwrap() - minscale) * maxcolor / range)
                            .clamp(0, 255) as usize;
                        cmap[idx]
                    })
                    .collect(),
            }
        } else {
            let invgamma = 1.0 / gamma;
            FrameData::<RGBAPixel> {
                width: self.width,
                height: self.height,
                data: self
                    .data
                    .iter()
                    .map(|x| {
                        let scaled = (((*x).to_i64().unwrap() - minscale) as f64 / range as f64)
                            .clamp(0.0, 1.0);
                        let scaled = f64::powf(scaled, invgamma);
                        let idx = (scaled * maxcolor as f64).clamp(0.0, 255.0) as usize;
                        cmap[idx]
                    })
                    .collect(),
            }
        }
    }
}
