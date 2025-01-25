/// A trait for pixel types
pub trait Pixel: Sized + Clone + Copy + std::fmt::Debug + Send + Sync + 'static + Default {}

/// A trait for pixels that are monochromatic.
/// These are generally pixels of type u8, u16, u32, etc...
pub trait MonoPixel:
    Pixel + num_traits::PrimInt + std::fmt::Debug + Send + Sync + 'static + Default
{
}
impl<T> Pixel for T where T: MonoPixel {}
impl<T> MonoPixel for T where
    T: num_traits::PrimInt + std::fmt::Debug + Send + Sync + 'static + Default
{
}

/// A pixel with red, green, and blue channels.
/// Each channel is an 8-bit unsigned integer.
#[derive(Clone, Copy, Debug, Default)]
pub struct RGBPixel {
    /// The red channel.
    pub r: u8,
    /// The green channel.
    pub g: u8,
    /// The blue channel.
    pub b: u8,
}

/// A pixel with red, green, blue, and alpha channels.
/// Each channel is an 8-bit unsigned integer.
#[derive(Clone, Copy, Debug)]
pub struct RGBAPixel {
    /// The red channel.
    pub r: u8,
    /// The green channel.
    pub g: u8,
    /// The blue channel.
    pub b: u8,
    /// The alpha channel.
    pub a: u8,
}

impl Pixel for RGBPixel {}
impl Pixel for RGBAPixel {}

impl Default for RGBAPixel {
    fn default() -> Self {
        RGBAPixel {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }
}
