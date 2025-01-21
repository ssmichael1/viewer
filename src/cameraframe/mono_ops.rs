//! This module contains the implementation of the arithmetic operations for the `FrameData` struct.

use super::FrameData;
use super::MonoPixel;

impl<T> FrameData<T>
where
    T: MonoPixel,
{
    pub fn zeros(width: u32, height: u32) -> FrameData<T> {
        FrameData::<T> {
            width,
            height,
            data: vec![T::zero(); (width * height) as usize],
        }
    }

    pub fn ones(width: u32, height: u32) -> FrameData<T> {
        FrameData::<T> {
            width,
            height,
            data: vec![T::one(); (width * height) as usize],
        }
    }
}

impl<T> std::ops::Shl<usize> for &FrameData<T>
where
    T: MonoPixel,
{
    type Output = FrameData<T>;

    fn shl(self, shift: usize) -> FrameData<T> {
        FrameData::<T> {
            width: self.width,
            height: self.height,
            data: self.data.iter().map(|a| *a << shift).collect(),
        }
    }
}

impl<T> std::ops::Shr<usize> for &FrameData<T>
where
    T: MonoPixel,
{
    type Output = FrameData<T>;

    fn shr(self, shift: usize) -> FrameData<T> {
        FrameData::<T> {
            width: self.width,
            height: self.height,
            data: self.data.iter().map(|a| *a >> shift).collect(),
        }
    }
}

impl<T> std::ops::ShlAssign<usize> for FrameData<T>
where
    T: MonoPixel,
{
    fn shl_assign(&mut self, shift: usize) {
        self.data.iter_mut().for_each(|a| *a = *a << shift);
    }
}

impl<T> std::ops::ShrAssign<usize> for FrameData<T>
where
    T: MonoPixel,
{
    fn shr_assign(&mut self, shift: usize) {
        self.data.iter_mut().for_each(|a| *a = *a >> shift);
    }
}

impl<T, T2> std::ops::Add<&FrameData<T2>> for &FrameData<T>
where
    T: MonoPixel,
    T2: MonoPixel,
{
    type Output = FrameData<T>;

    fn add(self, other: &FrameData<T2>) -> FrameData<T> {
        assert_eq!(self.width, other.width);
        assert_eq!(self.height, other.height);

        FrameData::<T> {
            width: self.width,
            height: self.height,
            data: self
                .data
                .iter()
                .zip(other.data.iter())
                .map(|(a, b)| *a + T::from(*b).unwrap())
                .collect(),
        }
    }
}

impl<T, T2> std::ops::AddAssign<&FrameData<T2>> for FrameData<T>
where
    T: MonoPixel,
    T2: MonoPixel,
{
    fn add_assign(&mut self, other: &FrameData<T2>) {
        assert_eq!(self.width, other.width);
        assert_eq!(self.height, other.height);

        self.data
            .iter_mut()
            .zip(other.data.iter())
            .for_each(|(a, b)| *a = *a + T::from(*b).unwrap());
    }
}

impl<T, T2> std::ops::Sub<&FrameData<T2>> for &FrameData<T>
where
    T: MonoPixel,
    T2: MonoPixel,
{
    type Output = FrameData<T>;

    fn sub(self, other: &FrameData<T2>) -> FrameData<T> {
        assert_eq!(self.width, other.width);
        assert_eq!(self.height, other.height);

        FrameData::<T> {
            width: self.width,
            height: self.height,
            data: self
                .data
                .iter()
                .zip(other.data.iter())
                .map(|(a, b)| *a - T::from(*b).unwrap())
                .collect(),
        }
    }
}

impl<T, T2> std::ops::SubAssign<&FrameData<T2>> for FrameData<T>
where
    T: MonoPixel,
    T2: MonoPixel,
{
    fn sub_assign(&mut self, other: &FrameData<T2>) {
        assert_eq!(self.width, other.width);
        assert_eq!(self.height, other.height);

        self.data
            .iter_mut()
            .zip(other.data.iter())
            .for_each(|(a, b)| *a = *a - T::from(*b).unwrap());
    }
}

impl<T, T2> std::ops::Mul<T2> for &FrameData<T>
where
    T: MonoPixel,
    T2: MonoPixel,
{
    type Output = FrameData<T>;
    fn mul(self, other: T2) -> FrameData<T> {
        let othert = T::from(other).unwrap();
        FrameData::<T> {
            width: self.width,
            height: self.height,
            data: self.data.iter().map(|a| *a * othert).collect(),
        }
    }
}

impl<T> std::ops::Div<T> for &FrameData<T>
where
    T: MonoPixel,
{
    type Output = FrameData<T>;
    fn div(self, other: T) -> FrameData<T> {
        let othert = T::from(other).unwrap();
        FrameData::<T> {
            width: self.width,
            height: self.height,
            data: self.data.iter().map(|a| *a / othert).collect(),
        }
    }
}

impl<T> std::ops::Mul<&FrameData<T>> for &FrameData<T>
where
    T: MonoPixel,
{
    type Output = FrameData<T>;

    fn mul(self, other: &FrameData<T>) -> FrameData<T> {
        assert_eq!(self.width, other.width);
        assert_eq!(self.height, other.height);

        FrameData::<T> {
            width: self.width,
            height: self.height,
            data: self
                .data
                .iter()
                .zip(other.data.iter())
                .map(|(a, b)| *a * *b)
                .collect(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_zeros() {
        let frame = FrameData::<u16>::zeros(3, 3);
        assert_eq!(frame.width, 3);
        assert_eq!(frame.height, 3);
        assert_eq!(frame.data, vec![0; 9]);
    }

    #[test]
    fn test_ones() {
        let frame = FrameData::<u16>::ones(3, 3);
        assert_eq!(frame.width, 3);
        assert_eq!(frame.height, 3);
        assert_eq!(frame.data, vec![1; 9]);
    }

    #[test]
    fn test_add() {
        let frame1 = FrameData::<u16>::ones(3, 3);
        let frame2 = FrameData::<u16>::ones(3, 3);
        let frame3 = &frame1 + &frame2;
        assert_eq!(frame3.width, 3);
        assert_eq!(frame3.height, 3);
        assert_eq!(frame3.data, vec![2; 9]);
    }

    #[test]
    fn test_add_types() {
        let frame1 = FrameData::<u16>::ones(3, 3);
        let frame2 = FrameData::<u8>::ones(3, 3);
        let frame3 = &frame1 + &frame2;
        assert_eq!(frame3.width, 3);
        assert_eq!(frame3.height, 3);
        assert_eq!(frame3.data, vec![2; 9]);
    }
}
