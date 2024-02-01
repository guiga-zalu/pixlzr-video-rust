use core::ops::{Add, Div, Mul, Neg, Sub};
use num_traits::One;

pub enum Operation {
    Width,
    Height,
    Scale,
    Duration,
    Framerate,
    Pixel,
}

// struct TransformationMatrix<T> {
//     translation: (T, T),
// }

impl<Rhs> Mul<Rhs> for Operation {
    type Output = (Self, Rhs);
    fn mul(self, rhs: Rhs) -> Self::Output {
        (self, rhs)
    }
}

impl<Rhs, U> Div<Rhs> for Operation
where
    Rhs: Div<Rhs, Output = U> + One + Mul<Rhs, Output = Rhs>,
{
    type Output = (Self, U);
    fn div(self, rhs: Rhs) -> Self::Output {
        let one: Rhs = One::one();
        (self, one / rhs)
    }
}
