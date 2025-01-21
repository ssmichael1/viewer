mod parula;

use crate::cameraframe::RGBAPixel;
use std::sync::OnceLock;

/// A colormap in our case is always 256 colors.
pub type ColorMap = [RGBAPixel; 256];

pub fn from_string(name: &str) -> Option<&'static ColorMap> {
    match name {
        "Parula" => Some(parula()),
        "Hot" => Some(hot()),
        "Grayscale" => Some(grayscale()),
        "Red" => Some(red()),
        _ => None,
    }
}

/// Create a grayscale colormap.
/// # Returns
/// A grayscale colormap.
pub fn grayscale() -> &'static ColorMap {
    static CMAP: OnceLock<ColorMap> = OnceLock::new();
    CMAP.get_or_init(|| {
        let mut colormap = [RGBAPixel {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }; 256];
        for (i, pixel) in colormap.iter_mut().enumerate() {
            *pixel = RGBAPixel {
                r: i as u8,
                g: i as u8,
                b: i as u8,
                a: 255,
            };
        }
        colormap
    })
}

pub fn red() -> &'static ColorMap {
    static CMAP: OnceLock<ColorMap> = OnceLock::new();
    CMAP.get_or_init(|| {
        let mut colormap = [RGBAPixel {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }; 256];
        for (i, pixel) in colormap.iter_mut().enumerate() {
            *pixel = RGBAPixel {
                r: i as u8,
                g: 0,
                b: 0,
                a: 255,
            };
        }
        colormap
    })
}

/// Create a "hot" colormap.
///
///  # Returns
/// A hot colormap.
///
pub fn hot() -> &'static ColorMap {
    static CMAP: OnceLock<ColorMap> = OnceLock::new();

    CMAP.get_or_init(|| {
        let mut colormap = [RGBAPixel {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }; 256];
        for (i, pixel) in colormap.iter_mut().take(128).enumerate() {
            *pixel = RGBAPixel {
                r: 0,
                g: (i * 2) as u8,
                b: 255,
                a: 255,
            };
        }
        for (i, pixel) in colormap.iter_mut().enumerate().skip(128) {
            *pixel = RGBAPixel {
                r: (i - 128) as u8,
                g: 255,
                b: 255 - (i - 128) as u8,
                a: 255,
            };
        }
        colormap
    })
}

/// The MATLAB colormap is called "parula"
pub use parula::parula;
