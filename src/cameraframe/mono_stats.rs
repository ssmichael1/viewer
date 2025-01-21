//!
//! This module contains functions for calculating statistics on FrameData with monochrome pixel values.
//!

use super::FrameData;
use super::MonoPixel;

impl<T> FrameData<T>
where
    T: MonoPixel,
{
    /// Calculate the mean of the data in the FrameData.
    ///
    /// # Returns
    /// The mean of the data in the FrameData.
    pub fn mean(&self) -> f64 {
        self.data.iter().fold(0, |acc, x| acc + x.to_i64().unwrap()) as f64 / self.data.len() as f64
    }

    /// Calculate the mean and variance of the data in the FrameData.
    ///
    /// # Returns
    /// A tuple containing the mean and variance of the data in the FrameData.
    ///
    pub fn mean_and_var(&self) -> (f64, f64) {
        let (sum, sumsq) = self.data.iter().fold((0, 0), |acc, x| {
            let (sum, sum_sq) = acc;
            let x = x.to_i64().unwrap();
            (sum + x, sum_sq + x * x)
        });
        let mean = sum as f64 / self.data.len() as f64;
        (mean, sumsq as f64 / self.data.len() as f64 - mean * mean)
    }

    /// Calculate the sum of the data in the FrameData.
    ///
    /// # Returns
    /// The sum of the data in the FrameData.
    ///
    pub fn sum(&self) -> i64 {
        self.data.iter().fold(0, |acc, x| acc + x.to_i64().unwrap())
    }

    /// Calculate the sum of the squares of the data in the FrameData.
    ///
    /// # Returns
    /// The sum of the squares of the data in the FrameData.
    ///
    pub fn sumsq(&self) -> i64 {
        self.data.iter().fold(0, |acc, x| {
            let x = x.to_i64().unwrap();
            acc + x * x
        })
    }

    /// Calculate the maximum value of the data in the FrameData.
    ///
    /// # Returns
    /// The maximum value of the data in the FrameData.
    pub fn maxval(&self) -> T {
        self.data
            .iter()
            .fold(T::min_value(), |acc, x| if *x > acc { *x } else { acc })
    }

    /// Calculate the minimum value of the data in the FrameData.
    ///
    /// # Returns
    /// The minimum value of the data in the FrameData.
    pub fn minval(&self) -> T {
        self.data
            .iter()
            .fold(T::max_value(), |acc, x| if *x < acc { *x } else { acc })
    }

    /// Calculate the minimum and maximum values of the data in the FrameData.
    pub fn minmax(&self) -> (T, T) {
        let mut min = T::max_value();
        let mut max = T::min_value();

        for x in self.data.iter() {
            if *x < min {
                min = *x;
            }
            if *x > max {
                max = *x;
            }
        }
        (min, max)
    }

    /// Used only for testing:
    ///
    /// Generate a FrameData with random values drawn from a normal distribution.
    ///
    /// # Arguments
    /// `mean` - The mean of the normal distribution
    /// `std_dev` - The standard deviation of the normal distribution
    /// `width` - The width of the FrameData
    /// `height` - The height of the FrameData
    ///
    /// # Returns
    /// A FrameData with random values drawn from a normal distribution.
    ///
    #[cfg(test)]
    pub fn rand_norm(mean: f64, std_dev: f64, width: u32, height: u32) -> FrameData<T> {
        use rand::distributions::Distribution;
        use rand_distr::Normal;

        let normal = Normal::new(mean, std_dev).unwrap();
        let mut rng = rand::thread_rng();

        FrameData::<T> {
            width,
            height,
            data: (0..width * height)
                .map(|_| T::from(normal.sample(&mut rng)).unwrap())
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean() {
        let frame = FrameData::<u8> {
            width: 3,
            height: 3,
            data: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
        };
        assert_eq!(frame.mean(), 5.0);
    }

    #[test]
    fn test_mean_and_var() {
        let frame = FrameData::<u8> {
            width: 3,
            height: 3,
            data: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
        };
        let (mean, var) = frame.mean_and_var();
        assert_eq!(mean, 5.0);
        assert!((var - 6.666666666666667).abs() < 1e-6);
    }

    #[test]
    fn test_large_mean_and_var() {
        let frame = FrameData::<u16>::rand_norm(1000.0, 100.0, 1000, 1000);
        let (mean, var) = frame.mean_and_var();
        println!("mean: {}, var: {}", mean, var);
        assert!((mean - 1000.0).abs() < 5.0);
        assert!((var - 10000.0).abs() < 40.0);
    }
}
