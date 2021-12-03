//! Game abstractions module.

pub use grid::*;

use crate::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::{error::Error, fmt::Display};

/// Data which's sent and recieved from game server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GameData {
	grid: Grid,
	snakes: Vec<Snake>,
	apples: Vec<Apple>,
}

impl GameData {
	/// Default size of the [`game grid`](Grid). Used when one isn't provided to the [`new`](Self::new)
	/// function or in the [`Default`](Self::default) implementation.
	pub const DEFAULT_GRID_SIZE: (usize, usize) = (100, 100);

	/// Recommended maximum number of snakes in the game.
	pub const RECOMMENDED_SNAKES_AMOUNT: usize = 5;

	/// Recommended maximum number of apples in the game.
	pub const RECOMMENDED_APPLES_AMOUNT: usize = 5;

	/// Default snake step. Should be changed only with purposes of fun.
	const SNAKE_STEP: i32 = 1;

	/// Return a new [`GameData`].
	///
	/// `snakes_max_amount` and `apples_max_amount` are maximum amounts of
	/// snakes and apples that can be in the game. If they're not specified,
	/// there can be unlimited amount of snakes or apples in the game. It's
	/// really recommended to specify these arguments to some value because else
	/// game can be slowed down when new snake or apple is added because vector
	/// of snakes and apples would be reallocated each time it happens.
	pub fn new(
		grid_size: (usize, usize),
		snakes_max_amount: Option<usize>,
		apples_max_amount: Option<usize>,
	) -> Self {
		Self {
			grid: Grid::new(grid_size),
			snakes: match snakes_max_amount {
				Some(val) => Vec::with_capacity(val),
				None => Vec::new(),
			},
			apples: match apples_max_amount {
				Some(val) => Vec::with_capacity(val),
				None => Vec::new(),
			},
		}
	}

	/// Refills [`game grid`](Grid) with a new data.
	pub fn update_grid(&mut self) {
		let mut grid = Grid::new(self.grid.size);
		for snake in &mut self.snakes {
			snake.move_parts(Self::SNAKE_STEP);
			for snake_part in &mut snake.parts {
				grid.data.push(GridPoint::new(
					GameObject::SnakePart,
					snake_part.coords(),
					snake_part.color(),
				));
			}
		}
		for apple in &self.apples {
			grid.data.push(GridPoint::new(
				GameObject::Apple,
				apple.coords(),
				Color::new(1.0, 0.0, 0.0, 0.0),
			))
		}
		self.grid = grid;
	}

	/// Add a new snake to the game. "coords" is a coordinates of leading
	/// part of a snake.
	pub fn spawn_snake<T: Into<String>>(
		&mut self,
		name: T,
		coords: Coordinates,
		direction: Direction,
		length: u32,
	) -> crate::Result<()> {
		let capacity = self.snakes.capacity();
		if capacity != 0 && capacity == self.snakes.len() {
			Err(Box::new(GameError::TooMuchSnakes))
		} else {
			self.snakes
				.push(Snake::new(name.into(), coords, direction, length));
			Ok(())
		}
	}

	/// Remove snake from the game and return it.
	pub fn kill_snake<T: Into<String>>(&mut self, name: T) -> crate::Result<Snake> {
		let name = name.into();
		match self.snakes.iter().position(|s| s.name() == name) {
			Some(index) => Ok(self.snakes.remove(index)),
			None => Err(Box::new(GameError::SnakeNotFound(name))),
		}
	}

	/// Return mutable reference to snake with specified name.
	pub fn snake<T: Into<String>>(&mut self, name: T) -> crate::Result<&mut Snake> {
		let name = name.into();
		for snake in &mut self.snakes.iter_mut() {
			if name == snake.name {
				return Ok(snake);
			}
		}
		Err(Box::new(GameError::SnakeNotFound(name)))
	}

	/// Return a vector of tuples with snake names and their lengths.
	pub fn scoreboard(&self) -> Vec<(String, usize)> {
		let mut scoreboard: Vec<(String, usize)> = Vec::with_capacity(self.snakes.len());
		for snake in &self.snakes {
			scoreboard.push((snake.name.clone(), snake.parts.len()))
		}
		scoreboard
	}

	/// Return game [`Grid`].
	pub fn grid(&self) -> Grid {
		self.grid.clone()
	}

	/// Convert [`GameData`] to binary json.
	pub fn as_bytes(&self) -> Result<Vec<u8>> {
		Ok(serde_json::to_string(self)?.as_bytes().to_vec())
	}

	/// Convert json string to [`GameData`].
	pub fn from_string<T: AsRef<str>>(string: T) -> Result<Self> {
		Ok(serde_json::from_str(string.as_ref())?)
	}
}

impl Default for GameData {
	/// Return a new [`GameData`] with possible unlimited amount of snake or
	/// apples in the game and grid size depending on
	/// [DEFAULT_GRID_SIZE](Self::DEFAULT_GRID_SIZE) constant.
	fn default() -> Self {
		Self::new(Self::DEFAULT_GRID_SIZE, None, None)
	}
}

/// Snake abstraction structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Snake {
	name: String,
	parts: Vec<SnakePart>,
}

impl Snake {
	/// Return [`Snake`] with specified name, initial leading part location,
	/// direction and length (amount of parts).
	fn new<T: Into<String>>(
		name: T,
		coordinates: Coordinates,
		direction: Direction,
		length: u32,
	) -> Self {
		let mut snake = Self {
			name: name.into(),
			parts: {
				let mut v = vec![];
				for i in 0..length {
					let offset = -(length as i32 + i as i32);
					let part_coords = match direction {
						Direction::Right => (coordinates.x + offset, coordinates.y),
						Direction::Left => (coordinates.x - offset, coordinates.y),
						Direction::Up => (coordinates.x, coordinates.y + offset),
						Direction::Down => (coordinates.x, coordinates.y - offset),
					}.into();
					v.push(SnakePart::new(
						part_coords,
						Color::GREEN,
						Direction::Right,
					));
				}
				v
			},
		};
		if let Some(lp) = snake.lp_mut() {
			lp.change_direction(direction);
		}
		snake
	}

	/// Relatively move all parts of the snake on `step` steps depending on its leading
	/// part direction.
	fn move_parts(&mut self, step: i32) {
		let parts = &mut self.parts;
		parts.reverse();
		for i in 0..parts.len() {
			let part = &mut parts[i].clone();
			match parts.get_mut(i + 1) {
				Some(next_part) => match part.direction {
					Direction::Up => next_part.mv((0, step)),
					Direction::Down => next_part.mv((0, -step)),
					Direction::Left => next_part.mv((-step, 0)),
					Direction::Right => next_part.mv((step, 0)),
				},
				None => break,
			};
		}
		parts.reverse();
		self.parts = parts.clone();
	}

	/// Check if snake is alive.
	///
	/// Return `true`, if it is, or `false`, if it's not.
	fn alive(&self) -> bool {
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

	/// Return immutable reference of the snake leading part.
	fn lp(&self) -> Option<&SnakePart> {
		self.parts.last()
	}

	/// Return mutable reference of the snake leading part.
	fn lp_mut(&mut self) -> Option<&mut SnakePart> {
		self.parts.last_mut()
	}

	/// Return snake parts without the leading one.
	fn pwl(&self) -> Vec<SnakePart> {
		let mut parts = self.parts.clone();
		parts.pop();
		parts
	}

	/// Change direction of the snake leading part. In other words, change snake
	/// direction.
	///
	/// # Panic
	/// This function will panic if there's no parts in the snake, e. g. if
	/// snake isn't alive.
	pub fn change_direction(&mut self, direction: Direction) -> crate::Result<()> {
		match self.lp_mut() {
			Some(lp) => {
				lp.change_direction(direction);
				Ok(())
			}
			None => panic!("there's no parts in the snake"),
		}
	}

	/// Return snake name.
	fn name(&self) -> String {
		self.name.clone()
	}
}

/// Snake part abstraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct SnakePart {
	coordinates: Coordinates,
	color: Color,
	#[serde(skip)]
	direction: Direction,
}

impl SnakePart {
	/// Return new part of a snake with specified coordinates, color, and direction.
	fn new(coordinates: Coordinates, color: Color, direction: Direction) -> Self {
		Self {
			coordinates,
			color,
			direction,
		}
	}

	/// Move part relative to current coordinates.
	///
	/// # Example
	/// ```
	/// // Create new part with (3, 4) coordinates.
	/// let mut part = SnakePart::new(Coordinates::new(3, 4), Color::BLACK, Direction::Right);
	///
	/// // Move part to (-5, 10) relative to its current coordinates.
	/// part.mv((-5, 10))
	///
	/// assert_eq!((-2, 14), part.coords());
	/// ```
	fn mv(&mut self, coordinates: (i32, i32)) {
		self.coordinates = Coordinates::new(
			self.coordinates.x + coordinates.0,
			self.coordinates.y + coordinates.1,
		)
	}

	/// Change part direction.
	fn change_direction(&mut self, direction: Direction) {
		self.direction = direction;
	}

	/// Return part coordinates.
	fn coords(&self) -> Coordinates {
		self.coordinates
	}

	/// Return part color.
	fn color(&self) -> Color {
		self.color
	}

	/// Set part coordinates.
	fn set_coords(&mut self, coordinates: Coordinates) {
		self.coordinates = coordinates;
	}
}

/// Apple which is going to be eaten by a snake.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct Apple {
	coordinates: Coordinates,
}

impl Apple {
	/// Return a new [`Apple`].
	fn new(coordinates: Coordinates) -> Self {
		Self { coordinates }
	}

	/// Return apple coordinates.
	fn coords(&self) -> Coordinates {
		self.coordinates
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Coordinates abstraction.
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

impl From<(i32, i32)> for Coordinates {
	fn from(t: (i32, i32)) -> Self {
		Self::new(t.0, t.1)
	}
}

/// Structure which determines direction of something.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// A color in the sRGB color space.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Color {
	/// Red component, 0.0 - 1.0
	pub r: f32,
	/// Green component, 0.0 - 1.0
	pub g: f32,
	/// Blue component, 0.0 - 1.0
	pub b: f32,
	/// Transparency, 0.0 - 1.0
	pub a: f32,
}

impl Color {
	/// The black color.
	pub const BLACK: Color = Color {
		r: 0.0,
		g: 0.0,
		b: 0.0,
		a: 1.0,
	};

	/// The white color.
	pub const WHITE: Color = Color {
		r: 1.0,
		g: 1.0,
		b: 1.0,
		a: 1.0,
	};

	/// The green color.
	pub const GREEN: Color = Color {
		r: 0.0,
		g: 1.0,
		b: 0.0,
		a: 1.0,
	};

	/// A color with no opacity.
	pub const TRANSPARENT: Color = Color {
		r: 0.0,
		g: 0.0,
		b: 0.0,
		a: 0.0,
	};

	/// Creates a new [`Color`].
	///
	/// In debug mode, it will panic if the values are not in the correct
	/// range: 0.0 - 1.0
	pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
		debug_assert!((0.0..=1.0).contains(&r), "Red component must be on [0, 1]");
		debug_assert!(
			(0.0..=1.0).contains(&g),
			"Green component must be on [0, 1]"
		);
		debug_assert!((0.0..=1.0).contains(&b), "Blue component must be on [0, 1]");
		debug_assert!(
			(0.0..=1.0).contains(&a),
			"Alpha component must be on [0, 1]"
		);

		Color { r, g, b, a }
	}

	/// Creates a [`Color`] from its RGB components.
	pub const fn from_rgb(r: f32, g: f32, b: f32) -> Color {
		Color::from_rgba(r, g, b, 1.0f32)
	}

	/// Creates a [`Color`] from its RGBA components.
	pub const fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
		Color { r, g, b, a }
	}

	/// Creates a [`Color`] from its RGB8 components.
	pub fn from_rgb8(r: u8, g: u8, b: u8) -> Color {
		Color::from_rgba8(r, g, b, 1.0)
	}

	/// Creates a [`Color`] from its RGB8 components and an alpha value.
	pub fn from_rgba8(r: u8, g: u8, b: u8, a: f32) -> Color {
		Color {
			r: f32::from(r) / 255.0,
			g: f32::from(g) / 255.0,
			b: f32::from(b) / 255.0,
			a,
		}
	}
}

/// Abstraction enum with available kinds of game objects.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameObject {
	/// A part of a snake.
	SnakePart,

	/// An apple.
	Apple,
}

/// Game grid abstractions.
pub mod grid {
	use rand::Rng;

use super::*;

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

		/// Change color of the [`GridPoint`]
		pub fn change_color(&mut self, color: Color) {
			self.color = color;
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
		pub const DEFAULT_SIZE: (usize, usize) = (10, 10);

		/// Return a new [`Grid`].
		pub fn new(size: (usize, usize)) -> Self {
			Self {
				data: Vec::with_capacity(size.0 * size.1),
				size,
			}
		}

		/// Return random coordinates fitting in the grid. Add offset to each
		/// randomly generated value, may be set to 0.
		pub fn random_coords(&self, offset: i32) -> Coordinates {
			let mut rng = rand::thread_rng();
			Coordinates::new(
				rng.gen_range(0..self.size.0) as i32 + offset,
				rng.gen_range(0..self.size.0) as i32 + offset,
			)
		}

		/// Convert [`Grid`] to binary json.
		pub fn as_bytes(&self) -> Result<Vec<u8>> {
			Ok(serde_json::to_string(self)?.as_bytes().to_vec())
		}
	}

	impl Default for Grid {
		fn default() -> Self {
			Self::new(Self::DEFAULT_SIZE)
		}
	}
}

/// Error type returned by [`game`](crate::game) module functions.
#[derive(Debug, Clone)]
pub enum GameError {
	/// Snake with name specified in argument name not found.
	SnakeNotFound(String),
	/// Adding a snake with name specified in variant argument when maximum
	/// amount of snakes in game is already reached.
	TooMuchSnakes,
}

impl Display for GameError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::SnakeNotFound(name) => write!(f, "snake with {} name not found", name),
			Self::TooMuchSnakes => write!(f, "maximum amount of snakes in the game is reached"),
		}
	}
}

impl Error for GameError {}
