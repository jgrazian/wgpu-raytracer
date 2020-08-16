use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[repr(C)]
#[derive(PartialEq, Default, Clone, Copy)]
pub struct Mat3 {
    x: [f32; 3],
    y: [f32; 3],
    z: [f32; 3],
}

impl Mat3 {
    pub fn new(
        a11: f32,
        a12: f32,
        a13: f32,
        a21: f32,
        a22: f32,
        a23: f32,
        a31: f32,
        a32: f32,
        a33: f32,
    ) -> Self {
        Mat3 {
            x: [a11, a12, a13],
            y: [a21, a22, a23],
            z: [a31, a32, a33],
        }
    }

    pub fn zero() -> Self {
        Mat3 {
            x: [0.0, 0.0, 0.0],
            y: [0.0, 0.0, 0.0],
            z: [0.0, 0.0, 0.0],
        }
    }
}
