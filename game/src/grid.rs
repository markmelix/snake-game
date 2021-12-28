//! Game grid abstractions.

use crate::{
    aux::{Color, Coordinates},
    Result,
};
/// Game grid abstractions.
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Abstraction enum with available kinds of game objects.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameObject {
    /// A part of a snake.
    SnakePart,

    /// An apple.
    Apple,
}

/// Struct which represents one unique point of the grid.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GridPoint {
    /// What kind of object is located in this point.
    pub object_kind: GameObject,

    /// [`Coordinates`] of the [`GridPoint`].
    /// Should be unique.
    pub coordinates: Coordinates,

    /// [`Color`] of the [`GridPoint`].
    pub color: Color,
}

impl GridPoint {
    /// Return a new [`GridPoint`].
    pub fn new(object_kind: GameObject, coordinates: Coordinates, color: Color) -> Self {
        Self {
            object_kind,
            coordinates,
            color,
        }
    }

    /// Change color of the [`GridPoint`].
    pub fn change_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Return coordinates of the [`GridPoint`].
    pub fn coords(&self) -> Coordinates {
        self.coordinates
    }
}

/// Game grid. In other words, vector of the [`GridPoint`]s.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    /// [`Grid`] data itself.
    pub data: Vec<GridPoint>,

    /// [`Grid`] size.
    pub size: (usize, usize),
}

impl Grid {
    /// Default size of the grid used with [`Default`](Self::default) trait
    /// implementation.
    pub const DEFAULT_SIZE: (usize, usize) = (50, 25);

    /// Return a new [`Grid`].
    pub fn new(size: (usize, usize)) -> Self {
        Self {
            data: Vec::with_capacity(size.0 * size.1),
            size,
        }
    }

    /// Generate random coordinates in range from (1 + `offset`) inclusively
    /// to (grid size - `offset`) exclusively and return them.
    ///
    /// `rng` is a random number generator, if it's `None`, then it's
    /// initialized automatically. This argument may be used if you have
    /// `rng` already initialized and you don't want to initialize it again.
    ///
    /// # Panic
    /// Panic if offset is less than any of grid sizes.
    pub fn random_coords(&self, offset: i32, rng: Option<rand::prelude::ThreadRng>) -> Coordinates {
        assert!(offset < self.size.0 as i32 && offset < self.size.1 as i32);
        let mut rng = rng.unwrap_or_default();
        Coordinates::new(
            rng.gen_range(1 + offset..=self.size.0 as i32 - offset) as i32,
            rng.gen_range(1 + offset..=self.size.1 as i32 - offset) as i32,
        )
    }

    /// Convert [`Grid`] to binary json.
    pub fn as_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_string(self)?.as_bytes().to_vec())
    }

    /// Convert json string to [`Grid`].
    pub fn from_string<T: AsRef<str>>(string: T) -> Result<Self> {
        Ok(serde_json::from_str(string.as_ref())?)
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new(Self::DEFAULT_SIZE)
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, point) in self.data.iter().enumerate() {
            writeln!(
                f,
                "{:?}[{}] at {} with rgba{} color",
                point.object_kind,
                i,
                point.coords(),
                point.color
            )?;
        }
        Ok(())
    }
}
