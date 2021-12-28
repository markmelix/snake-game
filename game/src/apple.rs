//! Apple abstractions.

use crate::aux::{Color, Coordinates};
use serde::{Deserialize, Serialize};

/// Apple which is going to be eaten by a snake.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Apple {
    coords: Coordinates,
    pub(crate) color: Color,
}

impl Apple {
    /// Default apple's color.
    pub const COLOR: Color = Color::RED;

    /// Return a new [`Apple`]. If `color` is none, use [`Self::COLOR`] one.
    pub(crate) fn new(coords: Coordinates, color: Option<Color>) -> Self {
        Self {
            coords,
            color: color.unwrap_or(Self::COLOR),
        }
    }

	/// Return apple's coordinates.
	pub(crate) fn coords(&self) -> Coordinates {
		self.coords
	}
}
