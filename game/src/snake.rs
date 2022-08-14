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
					let offset = i as i32;
					let part_coords = match direction {
						Direction::Right => {
							(coordinates.x + offset, coordinates.y)
						}
						Direction::Left => {
							(coordinates.x - offset, coordinates.y)
						}
						Direction::Up => {
							(coordinates.x, coordinates.y + offset)
						}
						Direction::Down => {
							(coordinates.x, coordinates.y - offset)
						}
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
	fn lp_move(&mut self, step: i32) -> Result<()> {
		let direction = self.direction;
		let lp = match self.lp_mut() {
			Some(lp) => lp,
			None => {
				return Err(Box::new(GameError::EmptySnake(self.name.clone())))
			}
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
					Err(Box::new(GameError::ChangeDirectionToOpposite(
						self.name(),
					)))
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
			let coords = match parts.get_mut(i + 1) {
				Some(next_part) => Some(next_part.coords()),
				None => break,
			};
			parts[i].set_coords(coords.unwrap());
		}
		self.lp_move(step)?;

		Ok(())
	}

	/// Check did some snake parts bump the leading one or not.
	///
	/// Return `true`, if they did, or `false`, if they didn't.
	pub(crate) fn parts_bumped(&self) -> Result<bool> {
		let lp = self.lp();
		if lp.is_none() {
			return Err(Box::new(GameError::EmptySnake(self.name())));
		}
		let lp = lp.unwrap();
		for part in self.pwl() {
			if part.coords() == lp.coords() {
				return Ok(true);
			}
		}
		Ok(false)
	}

	/// Incement snake size on `n` parts. If `colors` is none, then use snake's
	/// first part's color for all inserted parts, otherwise insert these parts
	/// reversed with colors in unwrapped `colors` vector.
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
	/// it being coordinated as a first snake's part. If it's none, then use
	/// snake's first part's color. To make this part coordinated as a tail of
	/// the snake you should run `self.move_parts` twice after running this
	/// method.
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
	fn pwl(&self) -> Vec<SnakePart> {
		let mut parts = self.parts.clone();
		parts.pop();
		parts
	}

	/// Return cloned snake name.
	pub(crate) fn name(&self) -> String {
		self.name.clone()
	}
}

/// Bump snake leading part with other ones. Needed for testing purposes.
#[allow(dead_code)]
pub(crate) fn bump_parts(snake: &mut Snake) -> Result<()> {
	snake.change_direction(Direction::Up)?;
	snake.move_parts(1)?;
	snake.change_direction(Direction::Left)?;
	snake.move_parts(1)?;
	snake.change_direction(Direction::Down)?;
	snake.move_parts(1)?;
	Ok(())
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

			if end == 0 || end < start {
				return Err(Box::new(ParseSnakeLengthError));
			}

			match inclusive {
				true => Ok(Self::Random(start..end + 1)),
				false => Ok(Self::Random(start..end)),
			}
		}
	}
}

/// Snake part abstraction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct SnakePart {
	coordinates: Coordinates,
	color: Color,
}

impl SnakePart {
	/// Return new part of a snake with specified coordinates and color.
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
	/// let mut part = SnakePart::new(Coordinates::new(3, 4), Color::BLACK);
	///
	/// // Move part to (-5, 10) relative to its current coordinates.
	/// part.mv((-5, 10));
	///
	/// assert_eq!((-2, 14), part.coords().into());
	/// ```
	fn mv(&mut self, coordinates: impl Into<Coordinates>) {
		self.coordinates = self.coordinates + coordinates.into();
	}

	/// Set part coordinates.
	fn set_coords(&mut self, coordinates: Coordinates) {
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

#[cfg(test)]
mod tests {
	use super::*;

	mod snake {
		use super::*;

		#[test]
		fn new() {
			let snake = Snake::new("snake", (0, 0).into(), Direction::Right, 5);

			assert_eq!(snake.name(), "snake".to_string());
			assert_eq!(snake.direction, Direction::Right);

			let part_coords = parts_into_tuple_coords(&snake.parts);

			assert_eq!(part_coords, [(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]);

			let snake = Snake::new("snake", (0, 0).into(), Direction::Left, 5);
			let part_coords = parts_into_tuple_coords(&snake.parts);

			assert_eq!(
				part_coords,
				[(0, 0), (-1, 0), (-2, 0), (-3, 0), (-4, 0)]
			);

			let snake = Snake::new("snake", (0, 0).into(), Direction::Up, 5);
			let part_coords = parts_into_tuple_coords(&snake.parts);

			assert_eq!(part_coords, [(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)]);

			let snake = Snake::new("snake", (0, 0).into(), Direction::Down, 5);
			let part_coords = parts_into_tuple_coords(&snake.parts);

			assert_eq!(
				part_coords,
				[(0, 0), (0, -1), (0, -2), (0, -3), (0, -4)]
			);
		}

		#[test]
		fn lp_move_expect_zero_length_error() {
			let mut snake =
				Snake::new("snake", (0, 0).into(), Direction::default(), 0);
			snake
				.lp_move(1)
				.expect_err("snake zero length error expected");
		}

		#[test]
		fn lp_move() -> Result<()> {
			let mut snake =
				Snake::new("snake", (0, 0).into(), Direction::Right, 5);
			snake.lp_move(1)?;
			let part_coords = parts_into_tuple_coords(&snake.parts);
			assert_eq!(part_coords, [(0, 0), (1, 0), (2, 0), (3, 0), (5, 0)]);

			let mut snake =
				Snake::new("snake", (0, 0).into(), Direction::Left, 5);
			snake.lp_move(-5)?;
			let part_coords = parts_into_tuple_coords(&snake.parts);
			assert_eq!(
				part_coords,
				[(0, 0), (-1, 0), (-2, 0), (-3, 0), (1, 0)]
			);

			let mut snake =
				Snake::new("snake", (0, 0).into(), Direction::Up, 5);
			snake.lp_move(5)?;
			let part_coords = parts_into_tuple_coords(&snake.parts);
			assert_eq!(part_coords, [(0, 0), (0, 1), (0, 2), (0, 3), (0, 9)]);

			Ok(())
		}

		#[test]
		fn change_dir() -> Result<()> {
			let mut snake = new_snake(Direction::Right, 1);
			snake.change_direction(Direction::Left)?;

			assert_eq!(snake.direction, Direction::Left);

			let mut snake = new_snake(Direction::Up, 5);
			snake
				.change_direction(Direction::Down)
				.expect_err("snake must have wanted to turn 180 degrees");

			Ok(())
		}

		#[test]
		fn move_parts() -> Result<()> {
			let mut snake = new_snake(Direction::Right, 5);
			snake.move_parts(1)?;
			let parts = parts_into_tuple_coords(&snake.parts);

			assert_eq!(parts, [(1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]);

			snake.change_direction(Direction::Up)?;
			snake.move_parts(1)?;
			let parts = parts_into_tuple_coords(&snake.parts);
			assert_eq!(parts, [(2, 0), (3, 0), (4, 0), (5, 0), (5, 1)]);

			snake.change_direction(Direction::Left)?;
			snake.move_parts(1)?;
			let parts = parts_into_tuple_coords(&snake.parts);
			assert_eq!(parts, [(3, 0), (4, 0), (5, 0), (5, 1), (4, 1)]);

			Ok(())
		}

		#[test]
		fn parts_bumped() -> Result<()> {
			let mut snake = new_snake(Direction::Right, 5);

			assert!(!snake.parts_bumped()?);
			bump_parts(&mut snake)?;
			assert!(snake.parts_bumped()?);

			Ok(())
		}

		#[test]
		fn insert_part() -> Result<()> {
			let mut snake = new_snake(Direction::Right, 5);
			snake.insert_part(Some(Color::WHITE))?;

			assert_eq!(
				SnakePart::new((0, 0).into(), Color::WHITE),
				snake.parts[0]
			);

			snake.move_parts(1)?;
			snake.move_parts(1)?;

			assert_eq!(
				SnakePart::new((1, 0).into(), Color::WHITE),
				snake.parts[0]
			);

			snake.insert_part(None)?;

			assert_eq!(snake.parts[0].color, Color::WHITE);

			Ok(())
		}

		#[test]
		fn inc_size() -> Result<()> {
			let mut snake = new_snake(Direction::Right, 5);
			let first_part_color = snake.parts[0].color; // save color of snake's first part

			assert_eq!(snake.len(), 5);

			snake.increment_size(5, None)?;

			assert_eq!(snake.len(), 10);

			// check that first five snake parts have same color as snake's old
			// first part
			assert_eq!(collect_colors(&snake.parts), [first_part_color; 5]);

			let mut check_colors = vec![Color::RED, Color::BLUE, Color::YELLOW];

			snake.increment_size(
				check_colors.len(),
				Some(check_colors.clone()),
			)?;

			check_colors.reverse();

			assert_eq!(
				collect_colors(&snake.parts)[0..check_colors.len()],
				check_colors
			);

			let mut check_colors = vec![Color::WHITE, Color::BLACK];

			snake.increment_size(3, Some(check_colors.clone()))?;

			check_colors.push(Color::BLACK);
			check_colors.reverse();

			assert_eq!(collect_colors(&snake.parts)[0..3], check_colors);

			Ok(())
		}

		#[test]
		fn len() {
			assert_eq!(new_snake(Default::default(), 18).len(), 18);
		}

		#[test]
		fn is_empty() {
			let snake = new_snake(Default::default(), 0);

			assert!(snake.is_empty());

			let snake = new_snake(Default::default(), 1);

			assert!(!snake.is_empty());
		}

		#[test]
		fn pwl() {
			let snake = new_snake(Default::default(), 5);
			assert_eq!(snake.pwl().last().unwrap().coords(), (3, 0).into());
		}

		/// Return a snake with (0, 0) leading part coordinates, `direction` and
		/// `n` parts.
		fn new_snake(direction: Direction, n: usize) -> Snake {
			Snake::new("snake", (0, 0).into(), direction, n)
		}

		/// Convert all SnakePart's vector into tuple's vector.
		fn parts_into_tuple_coords(parts: &[SnakePart]) -> Vec<(i32, i32)> {
			parts
				.iter()
				.map(|p| p.coords().into())
				.collect::<Vec<(i32, i32)>>()
		}

		fn collect_colors(parts: &[SnakePart]) -> Vec<Color> {
			parts[0..5].iter().map(|p| p.color).collect::<Vec<Color>>()
		}
	}

	mod snake_length {
		use super::*;

		#[test]
		fn snake_length() {
			assert_eq!(SnakeLength::Fixed(10).get(), 10);
			assert_eq!(<usize>::from(SnakeLength::Fixed(10)), 10);
			assert!(SnakeLength::Random(5..10).get() > 4);
			assert!(<SnakeLength>::from(5..10).get() > 4);
		}

		#[test]
		fn fromstr() -> Result<()> {
			assert_eq!(
				SnakeLength::Fixed(10).get(),
				"10".parse::<SnakeLength>()?.get()
			);
			assert!("5..10".parse::<SnakeLength>()?.get() < 10);
			assert!("5..10".parse::<SnakeLength>()?.get() > 4);
			assert!("5..=10".parse::<SnakeLength>()?.get() < 11);
			assert!("5..=10".parse::<SnakeLength>()?.get() > 4);

			assert!("asd".parse::<SnakeLength>().is_err());
			assert!("5.10".parse::<SnakeLength>().is_err());
			assert!("5=10".parse::<SnakeLength>().is_err());

			Ok(())
		}
	}

	mod snake_part {
		use super::*;

		#[test]
		fn mv() {
			let mut part = SnakePart::new(Coordinates::new(3, 4), Color::BLACK);

			part.mv((-5, 10));

			assert_eq!((-2, 14), part.coords().into());
		}
	}
}
