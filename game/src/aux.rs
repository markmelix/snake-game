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

impl ops::Sub for Coordinates {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.x - other.x, self.y - other.y)
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

    /// The red color.
    pub const RED: Color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };

    /// The yellow color.
    pub const YELLOW: Color = Color {
        r: 255,
        g: 255,
        b: 0,
        a: 255,
    };

	/// The magenta color.
    pub const MAGENTA: Color = Color {
        r: 255,
        g: 0,
        b: 255,
        a: 255,
    };

	/// The blue color.
    pub const BLUE: Color = Color {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };

	/// The cyan color.
    pub const CYAN: Color = Color {
        r: 0,
        g: 255,
        b: 255,
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

/// Like [`Vec::retain`], but retain between two values.
pub(crate) fn product_retain<T, F>(v: &mut Vec<T>, mut pred: F)
    where F: FnMut(&T, &T) -> bool
{
    let mut j = 0;
    for i in 0..v.len() {
        // invariants:
        // items v[0..j] will be kept
        // items v[j..i] will be removed
        if (0..j).chain(i + 1..v.len()).all(|a| pred(&v[i], &v[a])) {
            v.swap(i, j);
            j += 1;
        }
    }
    v.truncate(j);
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn coords_sum() {
		let coords1 = Coordinates::new(10, 15);
		let coords2 = Coordinates::new(-5, 3);
		let coords3 = Coordinates::new(5, 18);

		assert_eq!(coords1 + coords2, coords3);
	}

	#[test]
	fn coords_sub() {
		let coords1 = Coordinates::new(10, 15);
		let coords2 = Coordinates::new(-5, 3);
		let coords3 = Coordinates::new(15, 12);

		assert_eq!(coords1 - coords2, coords3);
	}

	#[test]
	fn dir_neg() {
		assert_eq!(Direction::Left, -Direction::Right);
		assert_eq!(Direction::Right, -Direction::Left);
		assert_eq!(Direction::Down, -Direction::Up);
		assert_eq!(Direction::Up, -Direction::Down);
	}

	#[test]
	fn dir_from_str() {
		assert_eq!(Direction::Up, "up".parse().unwrap());
		assert_eq!(Direction::Down, "down".parse().unwrap());
		assert_eq!(Direction::Left, "left".parse().unwrap());
		assert_eq!(Direction::Right, "right".parse().unwrap());
	}

	#[test]
	fn two_values_retain() {
		let mut vec = vec![1, 2, 3, 4, 5];
		product_retain(&mut vec, |a, b| a != b);
		assert_eq!(vec, [1, 2, 3, 4, 5]);
		
		let mut vec = vec![1, 2, 2, 3, 4, 5, 4, 1];
		product_retain(&mut vec, |a, b| a != b);
		assert_eq!(vec, [2, 3, 5, 4, 1]);
	}
}
