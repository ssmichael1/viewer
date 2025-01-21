/// A trait for pixel types
pub trait Pixel: Sized + Clone + Copy + std::fmt::Debug + Send + Sync + 'static {}

/// A trait for pixels that are monochromatic.
/// These are generally pixels of type u8, u16, u32, etc...
pub trait MonoPixel: Pixel + num_traits::PrimInt {}
impl<T> MonoPixel for T where T: Pixel + num_traits::PrimInt {}

/// A pixel with red, green, and blue channels.
/// Each channel is an 8-bit unsigned integer.
#[derive(Clone, Copy, Debug)]
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

/// Implement the Pixel trait for any type that implements num_traits::PrimInt and std::fmt::Debug.
impl<T> Pixel for T where T: num_traits::PrimInt + std::fmt::Debug + Send + Sync + 'static {}

/// Implement the pixel trait for the RGB pixel type.
impl Pixel for RGBPixel {}
/// Implement the pixel trait for the RGBA pixel type.
impl Pixel for RGBAPixel {}
