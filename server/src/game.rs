//! Game abstractions module.

pub use grid::*;

use crate::Result;
use serde::{Deserialize, Serialize};
use std::{error, fmt, ops};

/// Data which's sent and recieved from game server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GameData {
	grid: Grid,
	snakes: Vec<Snake>,
	apples: Vec<Apple>,
	snake_increment_size: usize,
}

impl GameData {
	/// Default size of the [`game grid`](Grid). Used when one isn't provided to the [`new`](Self::new)
	/// function or in the [`Default`](Self::default) implementation.
	pub const GRID_SIZE: (usize, usize) = Grid::DEFAULT_SIZE;

	/// Recommended maximum number of snakes in the game.
	pub const SNAKES_AMOUNT: usize = 5;

	/// Recommended maximum number of apples in the game.
	pub const APPLES_AMOUNT: usize = 1;

	/// Recommended snake increment size when it eats an apple.
	pub const SNAKE_INCREMENT_SIZE: usize = 1;

	/// Default snake step. Should be changed only with purpose of fun.
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
		grid_size: Option<(usize, usize)>,
		snakes_max_amount: Option<usize>,
		apples_max_amount: Option<usize>,
		snake_increment_size: Option<usize>,
	) -> Self {
		Self {
			grid: match grid_size {
				Some(val) => Grid::new(val),
				None => Grid::new(Self::GRID_SIZE),
			},
			snakes: match snakes_max_amount {
				Some(val) => Vec::with_capacity(val),
				None => Vec::new(),
			},
			apples: match apples_max_amount {
				Some(val) => Vec::with_capacity(val),
				None => Vec::new(),
			},
			snake_increment_size: match snake_increment_size {
				Some(val) => val,
				None => Self::SNAKE_INCREMENT_SIZE,
			},
		}
	}

	/// Kill over-bounded or bumped snakes.
	pub fn kill_dead_snakes(&mut self) {
		let snakes = self.snakes.clone();
		for mut i in 0..snakes.len() {
			if !&snakes[i].alive() {
				self.snakes.remove(i);
				continue;
			}
			for snake in &snakes {
				for part in &snake.pwl() {
					if self.snakes[i].lp().unwrap().coords() == part.coords() {
						self.snakes.remove(i);
						i -= 1;
					}
				}
			}
		}
	}

	/// Refill [`game grid`](Grid) with a new data and move all snakes.
	pub fn update_grid(&mut self) -> Result<()> {
		let mut grid = Grid::new(self.grid.size);
		for apple in &self.apples {
			grid.data.push(GridPoint::new(
				GameObject::Apple,
				apple.coords(),
				Color::RED,
			))
		}
		for snake in &mut self.snakes {
			snake.move_parts(Self::SNAKE_STEP)?;
			for snake_part in &mut snake.parts {
				grid.data.push(GridPoint::new(
					GameObject::SnakePart,
					snake_part.coords(),
					snake_part.color(),
				));
			}
		}
		self.grid = grid;
		Ok(())
	}

	/// Add a new snake to the game. "coords" is a coordinates of leading
	/// part of a snake.
	pub fn spawn_snake(
		&mut self,
		name: impl Into<String>,
		coords: Coordinates,
		direction: Direction,
		length: u32,
	) -> crate::Result<()> {
		let capacity = self.snakes.capacity();
		let name = name.into();
		if capacity != 0 && capacity == self.snakes.len() {
			Err(Box::new(GameError::TooMuchSnakes(name)))
		} else if self.find_snake(name.clone()) {
			Err(Box::new(GameError::NonUniqueName(name)))
		} else {
			self.snakes
				.push(Snake::new(name, coords, direction, length));
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

	/// Checks whether apples were eaten by snakes and if yes, increment number
	/// of their parts on `Self::snake_increment_size` ones and delete apples
	/// which were eaten. Spawn new apples if there're not any apples in the
	/// game.
	pub fn check_apples(&mut self) -> Result<()> {
		let mut delete_apples = Vec::with_capacity(self.apples.capacity());

		for snake in &mut self.snakes {
			if let Some(lp) = snake.lp() {
				let lp = lp.clone();
				for (i, apple) in self.apples.iter().enumerate() {
					if lp.coords() == apple.coords() {
						snake
							.increment_size(self.snake_increment_size, None)
							.unwrap();
						delete_apples.push(i);
					}
				}
			}
		}

		for index in delete_apples {
			self.apples.swap_remove(index);
		}

		while self.apples.len() < self.apples.capacity() {
			self.spawn_apple(self.grid.random_coords(0, None), None)?;
		}

		Ok(())
	}

	/// Return mutable reference to snake with specified name.
	pub fn snake_mut(&mut self, name: impl Into<String>) -> crate::Result<&mut Snake> {
		let name = name.into();
		for snake in &mut self.snakes {
			if name == snake.name {
				return Ok(snake);
			}
		}
		Err(Box::new(GameError::SnakeNotFound(name)))
	}

	/// Return immutable reference to snake with specified name.
	pub fn snake(&self, name: impl Into<String>) -> crate::Result<&Snake> {
		let name = name.into();
		for snake in &self.snakes {
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

	/// Return `true` if there's a snake with such `name` or `false` if there's not.
	pub fn find_snake(&self, name: impl Into<String>) -> bool {
		let name = name.into();
		for snake in &self.snakes {
			if snake.name() == name {
				return true;
			}
		}
		false
	}

	/// Add a new apple to the game. If `color` is none, use [`Apple::COLOR`]
	/// one.
	pub fn spawn_apple(&mut self, coords: Coordinates, color: Option<Color>) -> Result<()> {
		let capacity = self.apples.capacity();
		if capacity != 0 && capacity == self.apples.len() {
			Err(Box::new(GameError::TooMuchApples(coords)))
		} else {
			self.apples.push(Apple::new(coords, color));
			Ok(())
		}
	}

	/// Return number of snakes in the game.
	pub fn snakes(&self) -> usize {
		self.snakes.len()
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
	/// Return a new [`GameData`] with possible unlimited amount of snakes or
	/// apples in the game, grid size depending on
	/// [DEFAULT_GRID_SIZE](Self::DEFAULT_GRID_SIZE) constant and default
	/// snake's increment size.
	fn default() -> Self {
		Self::new(None, None, None, None)
	}
}

/// Snake abstraction structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Snake {
	name: String,
	parts: Vec<SnakePart>,

	/// Direction of snake's leading part.
	direction: Direction,
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
	fn step_move(&mut self, step: i32) -> Result<()> {
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
		match self.lp_mut() {
			Some(_) => {
				self.direction = direction;
				Ok(())
			}
			None => Err(Box::new(GameError::EmptySnake(self.name()))),
		}
	}

	/// Relatively move all parts of the snake on `step` steps depending on its
	/// leading part direction.
	fn move_parts(&mut self, step: i32) -> Result<()> {
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

	/// Incement snake size on `n` parts. If `colors` is none, then use snake's
	/// first part's color for all inserted parts, otherwise insert these parts
	/// with colors in unwrapped `colors` vector.
	fn increment_size(&mut self, mut n: usize, colors: Option<Vec<Color>>) -> Result<()> {
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
	fn insert_part(&mut self, color: Option<Color>) -> Result<()> {
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
}

impl SnakePart {
	/// Return new part of a snake with specified coordinates, color.
	fn new(coordinates: Coordinates, color: Color) -> Self {
		Self { coordinates, color }
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
	fn mv(&mut self, coordinates: impl Into<Coordinates>) {
		self.coordinates = self.coordinates + coordinates.into();
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
	color: Color,
}

impl Apple {
	/// Default apple's color.
	pub const COLOR: Color = Color::RED;

	/// Return a new [`Apple`]. If `color` is none, use [`Self::COLOR`] one.
	fn new(coordinates: Coordinates, color: Option<Color>) -> Self {
		Self {
			coordinates,
			color: color.unwrap_or(Self::COLOR),
		}
	}

	/// Return apple's coordinates.
	fn coords(&self) -> Coordinates {
		self.coordinates
	}

	/// Return apple's color.
	fn color(&self) -> Color {
		self.color
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Coordinates abstraction.
///
/// Note that this coordinates system is same as in math, so (0, 0) point is the
/// bottom left corner of the screen.
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
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
		pub fn random_coords(
			&self,
			offset: usize,
			rng: Option<rand::prelude::ThreadRng>,
		) -> Coordinates {
			let mut rng = rng.unwrap_or_default();
			Coordinates::new(
				rng.gen_range(1 + offset..=self.size.0 - offset) as i32,
				rng.gen_range(1 + offset..=self.size.1 - offset) as i32,
			)
		}

		/// Convert [`Grid`] to binary json.
		pub fn as_bytes(&self) -> Result<Vec<u8>> {
			Ok(serde_json::to_string(self)?.as_bytes().to_vec())
		}

		/// Convert json string to [`GameData`].
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
}

/// Error type returned by [`game`](crate::game) module functions.
#[derive(Debug, Clone)]
pub enum GameError {
	/// Snake with name specified in variant argument not found.
	SnakeNotFound(String),

	/// Adding a snake with name specified in variant argument when maximum
	/// amount of snakes in game is already reached.
	TooMuchSnakes(String),

	/// Adding an apple with coordinates specified in variant argument when
	/// maximum amount of apples in game is already reached.
	TooMuchApples(Coordinates),

	/// Snake with name specified in variant argument has no parts.
	EmptySnake(String),

	/// Snake with name specified in variant argument exists.
	NonUniqueName(String),
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
		}
	}
}

impl error::Error for GameError {}
