//! Auxiliary abstractions.

use crate::error::*;
use rand_derive2::RandGen;
use serde::{Deserialize, Serialize};
use std::{fmt, ops, str::FromStr};

/// Coordinates abstraction.
///
/// Note that this coordinates system is same as in math, so (0, 0) point is the
/// bottom left corner of the screen.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Coordinates {
    /// Coordinate relative to the abscissa axis.
    pub x: i32,

    /// Coordinate relative to the ordinate axis.
    pub y: i32,
}

impl Coordinates {
    /// Return a new [`Coordinates`].
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Convert [`Coordinates`] to array with two u32 elements.
    pub fn to_u32(self) -> [i32; 2] {
        [self.x, self.y]
    }

    /// Convert [`Coordinates`] to array with two f32 elements.
    pub fn to_f32(self) -> [f32; 2] {
        [self.x as f32, self.y as f32]
    }
}

impl From<Coordinates> for (i32, i32) {
	fn from(c: Coordinates) -> Self {
		(c.x, c.y)
	}
}

impl From<(i32, i32)> for Coordinates {
    fn from(t: (i32, i32)) -> Self {
        Self::new(t.0, t.1)
    }
}

impl ops::Add for Coordinates {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl fmt::Display for Coordinates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// Structure which determines direction of something.
#[derive(Debug, Clone, Copy, PartialEq, RandGen, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// Up.
    Up,

    /// Down.
    Down,

    /// Left.
    Left,

    /// Right.
    Right,
}

impl Direction {
    /// Return opposite direction.
    pub fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Self::Right
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lower_case = format!("{:?}", self).to_lowercase();
        write!(f, "{}", lower_case)
    }
}

impl FromStr for Direction {
    type Err = ParseDirectionError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "up" => Ok(Self::Up),
            "down" => Ok(Self::Down),
            "right" => Ok(Self::Right),
            "left" => Ok(Self::Left),
            _ => Err(ParseDirectionError),
        }
    }
}

impl ops::Neg for Direction {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.opposite()
    }
}

/// A color in the sRGB color space.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Color {
    /// Red component
    pub r: u8,

    /// Green component
    pub g: u8,

    /// Blue component
    pub b: u8,

    /// Transparency
    pub a: u8,
}

impl Color {
    /// The black color.
    pub const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };

    /// The white color.
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    /// The green color.
    pub const GREEN: Color = Color {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };

    /// The green color.
    pub const RED: Color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };

    /// A color with no opacity.
    pub const TRANSPARENT: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    /// Return a new [`Color`]
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }
}
