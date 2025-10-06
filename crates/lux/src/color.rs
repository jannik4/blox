use bevy_color::prelude::*;
use std::ops::{Add, AddAssign, Div, Mul};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct LinearRgb {
    /// The red channel. [0.0, 1.0]
    pub red: f32,
    /// The green channel. [0.0, 1.0]
    pub green: f32,
    /// The blue channel. [0.0, 1.0]
    pub blue: f32,
}

impl LinearRgb {
    pub const BLACK: Self = Self {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
    };

    pub const WHITE: Self = Self {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
    };

    pub fn new(red: f32, green: f32, blue: f32) -> Self {
        Self { red, green, blue }
    }
}

impl From<LinearRgba> for LinearRgb {
    fn from(value: LinearRgba) -> Self {
        Self {
            red: value.red,
            green: value.green,
            blue: value.blue,
        }
    }
}

impl From<LinearRgb> for LinearRgba {
    fn from(value: LinearRgb) -> Self {
        Self {
            red: value.red,
            green: value.green,
            blue: value.blue,
            alpha: 1.0,
        }
    }
}

impl From<Color> for LinearRgb {
    fn from(value: Color) -> Self {
        Self::from(LinearRgba::from(value))
    }
}

impl From<LinearRgb> for Color {
    fn from(value: LinearRgb) -> Self {
        Color::LinearRgba(LinearRgba::from(value))
    }
}

impl Mix for LinearRgb {
    #[inline]
    fn mix(&self, other: &Self, factor: f32) -> Self {
        let n_factor = 1.0 - factor;
        Self {
            red: self.red * n_factor + other.red * factor,
            green: self.green * n_factor + other.green * factor,
            blue: self.blue * n_factor + other.blue * factor,
        }
    }
}

impl Add<Self> for LinearRgb {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            red: self.red + rhs.red,
            green: self.green + rhs.green,
            blue: self.blue + rhs.blue,
        }
    }
}

impl AddAssign<Self> for LinearRgb {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Mul<Self> for LinearRgb {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self {
            red: self.red * rhs.red,
            green: self.green * rhs.green,
            blue: self.blue * rhs.blue,
        }
    }
}

impl Mul<f32> for LinearRgb {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self {
            red: self.red * rhs,
            green: self.green * rhs,
            blue: self.blue * rhs,
        }
    }
}

impl Mul<LinearRgb> for f32 {
    type Output = LinearRgb;

    fn mul(self, rhs: LinearRgb) -> LinearRgb {
        LinearRgb {
            red: self * rhs.red,
            green: self * rhs.green,
            blue: self * rhs.blue,
        }
    }
}

impl Div<f32> for LinearRgb {
    type Output = LinearRgb;

    fn div(self, rhs: f32) -> LinearRgb {
        LinearRgb {
            red: self.red / rhs,
            green: self.green / rhs,
            blue: self.blue / rhs,
        }
    }
}
