#![allow(unused_imports)]

mod enums;
pub mod short_names {
    pub use super::Operation::{
        Duration as D, Framerate as F, Height as H, Pixel as P, Scale as S, Width as W,
    };
}

use core::ops::Add;
pub use enums::*;
use image::DynamicImage;
use short_names::*;

pub struct Image(DynamicImage);

// impl<T> Add<(Operation, T)> for Image {
//     type Output = Image;
//     fn sub(self, (operation, rhs): (Operation, T)) -> Self::Output {
//         match operation {
//             D => unimplemented!(),
//             F => unimplemented!(),
//             H => unimplemented!(),
//             W => unimplemented!(),
//             S => unimplemented!(),
//             P => {
//                 unimplemented!()
//             }
//         }
//     }
// }
