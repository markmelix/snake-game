//! Errors returned by functions related to this crate.

use crate::aux::*;
use std::{error, fmt};

/// Error type returned by crate's functions.
#[derive(Debug, Clone)]
pub enum GameError {
	/// Snake with name specified in variant's argument not found.
	SnakeNotFound(String),

	/// Adding a snake with name specified in variant's argument when maximum
	/// amount of snakes in game is already reached.
	TooMuchSnakes(String),

	/// Adding an apple with coordinates specified in variant's argument when
	/// maximum amount of apples in game is already reached.
	TooMuchApples(Coordinates),

	/// Snake with name specified in variant's argument has no parts.
	EmptySnake(String),

	/// Snake with name specified in variant's argument exists.
	NonUniqueName(String),

	/// Snake with name specified in variant's argument and length greater than
	/// one tries to turn 180 degrees.
	ChangeDirectionToOpposite(String),
}

impl fmt::Display for GameError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
            Self::SnakeNotFound(name) => write!(f, "snake with {} name not found", name),
            Self::TooMuchSnakes(name) => write!(f,
				"can't add snake with name {} because maximum amount of snakes in the game is reached", name),
            Self::TooMuchApples(coords) => write!(f,
				"can't add apples with {} coords because maximum amount of apples in the game is reached", coords),
            Self::EmptySnake(name) => write!(f, "snake with {} name has no parts", name),
            Self::NonUniqueName(name) => write!(f, "snake with {} name already exists", name),
			Self::ChangeDirectionToOpposite(name) => write!(f, "snake with {} name tries to turn 180 degrees", name),
        }
	}
}

impl error::Error for GameError {}

/// Error returned if can't parse [`Direction`] from a string.
#[derive(Debug, Clone)]
pub struct ParseDirectionError;

impl fmt::Display for ParseDirectionError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f,
"can't parse Direction because parsed string is not \"up\", \"down\", \"left\", \"right\" or \"random\"")
	}
}

impl error::Error for ParseDirectionError {}

/// Error returned if can't parse [`SnakeLength`](crate::snake::SnakeLength) from a string.
#[derive(Debug, Clone)]
pub struct ParseSnakeLengthError;

impl fmt::Display for ParseSnakeLengthError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f,
"can't parse SnakeLength because parsed string is not a valid range or number")
	}
}

impl error::Error for ParseSnakeLengthError {}
