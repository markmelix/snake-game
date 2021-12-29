//! Snake abstractions.

use crate::{aux::*, error::*, Result};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{fmt, ops, str::FromStr};

/// Snake abstraction structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Snake {
    pub(crate) name: String,
    pub(crate) parts: Vec<SnakePart>,

    /// Direction of snake's leading part.
    pub(crate) direction: Direction,
}

impl Snake {
    /// Return [`Snake`] with specified name, initial leading part location,
    /// direction and length (amount of parts).
    pub(crate) fn new<T: Into<String>>(
        name: T,
        coordinates: Coordinates,
        direction: Direction,
        length: usize,
    ) -> Self {
        Self {
            name: name.into(),
            parts: {
                let mut v = vec![];
                for i in 0..length {
                    let offset = length as i32 + i as i32;
                    let part_coords = match direction {
                        Direction::Right => (coordinates.x + offset, coordinates.y),
                        Direction::Left => (coordinates.x - offset, coordinates.y),
                        Direction::Up => (coordinates.x, coordinates.y + offset),
                        Direction::Down => (coordinates.x, coordinates.y - offset),
                    }
                    .into();

                    let part_color = if i == length - 1 {
                        Color::new(0, 200, 0, 255)
                    } else {
                        Color::GREEN
                    };

                    v.push(SnakePart::new(part_coords, part_color));
                }
                v
            },
            direction,
        }
    }

    /// Move snake's leading part relatively to current direction on `step`
    /// points.
    pub(crate) fn step_move(&mut self, step: i32) -> Result<()> {
        let direction = self.direction;
        let lp = match self.lp_mut() {
            Some(lp) => lp,
            None => return Err(Box::new(GameError::EmptySnake(self.name.clone()))),
        };
        match direction {
            Direction::Up => lp.mv((0, step)),
            Direction::Down => lp.mv((0, -step)),
            Direction::Left => lp.mv((-step, 0)),
            Direction::Right => lp.mv((step, 0)),
        }
        Ok(())
    }

    /// Change snake's leading part direction.
    pub fn change_direction(&mut self, direction: Direction) -> Result<()> {
        match self.is_empty() {
            false => {
				if self.len() > 1 && self.direction == -direction {
					Err(Box::new(GameError::ChangeDirectionToOpposite(self.name())))
				} else {
					self.direction = direction;
					Ok(())
				}
            }
            true => Err(Box::new(GameError::EmptySnake(self.name()))),
        }
    }

    /// Relatively move all parts of the snake on `step` steps depending on its
    /// leading part direction.
    pub(crate) fn move_parts(&mut self, step: i32) -> Result<()> {
        let parts = &mut self.parts;

        for i in 0..parts.len() {
            let coords;
            match parts.get_mut(i + 1) {
                Some(next_part) => coords = Some(next_part.coords()),
                None => break,
            };
            parts[i].set_coords(coords.unwrap());
        }
        self.step_move(step)?;

        Ok(())
    }

    /// Check if snake is alive.
    ///
    /// Return `true`, if it is, or `false`, if it's not.
    pub(crate) fn alive(&self) -> bool {
        let lp = match self.lp() {
            Some(val) => val,
            None => return false,
        };
        for part in self.pwl() {
            if part.coords() == lp.coords() {
                return false;
            }
        }
        true
    }

    /// Incement snake size on `n` parts. If `colors` is none, then use snake's
    /// first part's color for all inserted parts, otherwise insert these parts
    /// with colors in unwrapped `colors` vector.
    pub(crate) fn increment_size(
        &mut self,
        mut n: usize,
        colors: Option<Vec<Color>>,
    ) -> Result<()> {
        if n == 0 {
            return Ok(());
        }
        match colors {
            Some(colors) => {
                for color in colors {
                    self.insert_part(Some(color))?;
                    n -= 1;
                }
                for _ in 0..n {
                    self.insert_part(None)?;
                }
            }
            None => {
                for _ in 0..n {
                    self.insert_part(None)?;
                }
            }
        }
        Ok(())
    }

    /// Insert part with `color` color into the start of parts vector and make
    /// it being coordinated as a tail of the snake. If it's none, then use
    /// snake's first part's color.
    pub(crate) fn insert_part(&mut self, color: Option<Color>) -> Result<()> {
        let tail_part = match self.parts.first() {
            Some(part) => part.clone(),
            None => return Err(Box::new(GameError::EmptySnake(self.name()))),
        };
        let color = match color {
            Some(color) => color,
            None => tail_part.color(),
        };

        self.parts
            .insert(0, SnakePart::new(tail_part.coords(), color));

        Ok(())
    }

	/// Return snake's length (amount of parts).
	pub fn len(&self) -> usize {
		self.parts.len()
	}

	/// Return true if snake has zero length, false otherwise.
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

    /// Return immutable reference of the snake leading part.
    pub(crate) fn lp(&self) -> Option<&SnakePart> {
        self.parts.last()
    }

    /// Return mutable reference of the snake leading part.
    pub(crate) fn lp_mut(&mut self) -> Option<&mut SnakePart> {
        self.parts.last_mut()
    }

    /// Return snake parts without the leading one.
    pub(crate) fn pwl(&self) -> Vec<SnakePart> {
        let mut parts = self.parts.clone();
        parts.pop();
        parts
    }

    /// Return snake name.
    pub(crate) fn name(&self) -> String {
        self.name.clone()
    }
}

/// Snake initial length abstraction.
#[derive(Debug, Clone)]
pub enum SnakeLength {
    /// Range to be used for generating random length.
    Random(ops::Range<usize>),

    /// Fixed length.
    Fixed(usize),
}

impl SnakeLength {
    pub fn get(self) -> usize {
        match self {
            Self::Random(range) => rand::thread_rng().gen_range(range),
            Self::Fixed(number) => number,
        }
    }
}

impl fmt::Display for SnakeLength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Random(range) => write!(f, "{:?}", range),
            Self::Fixed(n) => write!(f, "{}", n),
        }
    }
}

impl From<ops::Range<usize>> for SnakeLength {
    fn from(range: ops::Range<usize>) -> Self {
        Self::Random(range)
    }
}

impl From<SnakeLength> for usize {
    fn from(l: SnakeLength) -> Self {
        l.get()
    }
}

impl FromStr for SnakeLength {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self> {
        if let Ok(n) = s.parse::<usize>() {
            Ok(Self::Fixed(n))
        } else {
            let mut inclusive = false;
            let mut start = 0;
            let mut end = 0;

            for (i, token) in s.split("..").enumerate() {
                if i == 0 && token.parse::<usize>().is_ok() {
                    start = token.parse::<usize>().unwrap();
                } else if i == 1 {
                    if token.starts_with('=') {
                        inclusive = true;
                        end = token.get(1..).unwrap().parse::<usize>()?;
                    } else {
                        end = token.parse::<usize>()?;
                    }
                }
            }

            match inclusive {
                true => Ok(Self::Random(start..end + 1)),
                false => Ok(Self::Random(start..end)),
            }
        }
    }
}

/// Snake part abstraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct SnakePart {
    coordinates: Coordinates,
    color: Color,
}

impl SnakePart {
    /// Return new part of a snake with specified coordinates, color.
    pub(crate) fn new(coordinates: Coordinates, color: Color) -> Self {
        Self { coordinates, color }
    }

    /// Move part relative to current coordinates.
    ///
    /// # Example
    /// ```ignore
    /// use game::{snake::SnakePart, aux::*};
    /// 
    /// // Create new part with (3, 4) coordinates.
    /// let mut part = SnakePart::new(Coordinates::new(3, 4), Color::BLACK, Direction::Right);
    ///
    /// // Move part to (-5, 10) relative to its current coordinates.
    /// part.mv((-5, 10));
    ///
    /// assert_eq!((-2, 14).into(), part.coords());
    /// ```
    pub(crate) fn mv(&mut self, coordinates: impl Into<Coordinates>) {
        self.coordinates = self.coordinates + coordinates.into();
    }

    /// Set part coordinates.
    pub(crate) fn set_coords(&mut self, coordinates: Coordinates) {
        self.coordinates = coordinates;
    }
	
    /// Return part coordinates.
    pub(crate) fn coords(&self) -> Coordinates {
        self.coordinates
    }

    /// Return part color.
    pub(crate) fn color(&self) -> Color {
        self.color
    }
}
