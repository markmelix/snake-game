//! Game abstractions crate.

pub mod apple;
pub mod aux;
pub mod error;
pub mod grid;
pub mod snake;

/// This is an alias for standart [`Result`](std::result::Result) type which
/// represents failure.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Common reexports in one place.
pub mod prelude {
	pub use crate::{
		aux::*, grid::Grid, snake::SnakeLength, GameData, Settings,
	};
}

use apple::Apple;
use aux::{Color, Coordinates, Direction};
use error::GameError;
use grid::{GameObject, Grid, GridPoint};
use itertools::Itertools;
use snake::{Snake, SnakeLength};

/// Game settings and data.
#[derive(Debug, Clone, Default)]
pub struct GameData {
	grid: Grid,
	snakes: Vec<Snake>,
	apples: Vec<Apple>,
	settings: Settings,
}

impl GameData {
	/// Default size of the [`game grid`](Grid). Used when one isn't provided to the [`new`](Self::new)
	/// function or in the [`Default`](Self::default) implementation.
	pub const GRID_SIZE: (usize, usize) = Grid::DEFAULT_SIZE;

	/// Return a new [`GameData`].
	pub fn new(grid_size: Option<(usize, usize)>, settings: Settings) -> Self {
		Self {
			grid: Grid::new(grid_size.unwrap_or(Self::GRID_SIZE)),
			snakes: Vec::with_capacity(settings.clone().snakes_amount),
			apples: Vec::with_capacity(settings.clone().apples_amount),
			settings,
		}
	}

	/// Kill over-bounded or bumped snakes.
	pub fn kill_dead_snakes(&mut self) {
		let mut kill_queue = Vec::with_capacity(self.snakes());
		for snake in &self.snakes {
			if snake.parts_bumped().unwrap_or(true) || {
				let (w, h) = (self.grid.size.0 as i32, self.grid.size.1 as i32);
				let (x, y): (i32, i32) = snake.lp().unwrap().coords().into();

				x < 1 || x > w || y < 1 || y > h
			} {
				kill_queue.push(snake.name());				
			}
		}
		for perm in self.snakes.iter().permutations(2) {
			let (s1, s2) = (perm[0], perm[1]);
			if s1.name == s2.name || kill_queue.contains(&s1.name) || kill_queue.contains(&s2.name) {
				continue;
			}
			let s1_lp_coords = s1.lp().unwrap().coords();
		        for s2_part in &s2.parts {
					println!("{}: {}; {}: {}", s1.name(), s1_lp_coords, s2.name(), s2_part.coords());
		            if s1_lp_coords == s2_part.coords() {
		                kill_queue.push(s1.name());
		            }
		        }
		}
		self.snakes.retain(|snake| !kill_queue.contains(&snake.name));
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
			snake.move_parts(self.settings.snake_step)?;
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

	/// Add a new snake to the game. `coords` is a coordinates of leading part
	/// of a snake, if it's none, use random ones. If `length` is none, use one
	/// from the game settings. If direction is `Some(None)`, use random one,
	/// if it's `None`, use one from the game settings.
	pub fn spawn_snake(
		&mut self,
		name: impl Into<String>,
		coords: Option<Coordinates>,
		direction: Option<Option<Direction>>,
		length: Option<usize>,
	) -> crate::Result<()> {
		let capacity = self.snakes.capacity();
		let name = name.into();
		if capacity != 0 && capacity == self.snakes.len() {
			Err(Box::new(GameError::TooMuchSnakes(name)))
		} else if self.find_snake(name.clone()) {
			Err(Box::new(GameError::NonUniqueName(name)))
		} else {
			let direction = direction
				.unwrap_or(self.settings.snake_direction)
				.unwrap_or_else(rand::random);
			let length: usize = length
				.unwrap_or_else(|| self.settings.snake_length.clone().into());
			let coords = coords.unwrap_or_else(|| self.grid.random_coords());

			self.snakes
				.push(Snake::new(name, coords, direction, length));
			Ok(())
		}
	}

	/// Remove snake from the game and return it.
	pub fn kill_snake<T: Into<String>>(
		&mut self,
		name: T,
	) -> crate::Result<Snake> {
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
							.increment_size(
								self.settings.snake_increment_size,
								None,
							)
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
			self.spawn_apple(self.grid.random_coords(), None)?;
		}

		Ok(())
	}

	/// Return mutable reference to snake with specified name.
	pub fn snake_mut(
		&mut self,
		name: impl Into<String>,
	) -> crate::Result<&mut Snake> {
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
		let mut scoreboard: Vec<(String, usize)> =
			Vec::with_capacity(self.snakes.len());
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
	pub fn spawn_apple(
		&mut self,
		coords: Coordinates,
		color: Option<Color>,
	) -> Result<()> {
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

	/// Return game [`settings`](Settings).
	pub fn settings(&self) -> Settings {
		self.settings.clone()
	}
}

/// Game settings.
#[derive(Debug, Clone)]
pub struct Settings {
	/// Maximum number of snakes in the game.
	///
	/// If it's equals to zero, there can be unlimited amount of snakes in a
	/// game. It's really recommended to specify this argument to non-zero value
	/// because else game can be slowed down when a new snake is added because
	/// vector of snakes would be reallocated each time it happens.
	pub snakes_amount: usize,

	/// Maximum number of apples in the game.
	///
	/// If it's equals to zero, there can be unlimited amount of apples in a
	/// game. It's really recommended to specify this argument to non-zero value
	/// because else game can be slowed down when a new apple is added because
	/// vector of apples would be reallocated each time it happens.
	pub apples_amount: usize,

	/// How many steps snake does when it goes.
	pub snake_step: i32,

	/// How many parts should be added to snake when it eats an apple.
	pub snake_increment_size: usize,

	/// Initial snake length.
	pub snake_length: SnakeLength,

	/// Initial snake direction. If it's none, use random direction for every
	/// new snake.
	pub snake_direction: Option<Direction>,
}

impl Settings {
	/// Default maximum number of snakes in the game.
	pub const SNAKES_AMOUNT: usize = 5;

	/// Default maximum number of apples in the game.
	pub const APPLES_AMOUNT: usize = 1;

	/// Default snake increment size when it eats an apple.
	pub const SNAKE_INCREMENT_SIZE: usize = 1;

	/// Default snake length when it spawns.
	pub const SNAKE_LENGTH: SnakeLength = SnakeLength::Fixed(1);

	/// Default snake step. Should be changed only with purpose of fun.
	pub const SNAKE_STEP: i32 = 1;

	/// Default initial snake direction. If it's none, use random direction for
	/// every new snake.
	pub const SNAKE_DIRECTION: Option<Direction> = Some(Direction::Right);
}

impl Default for Settings {
	fn default() -> Self {
		Self {
			snakes_amount: Self::SNAKES_AMOUNT,
			apples_amount: Self::APPLES_AMOUNT,
			snake_step: Self::SNAKE_STEP,
			snake_increment_size: Self::SNAKE_INCREMENT_SIZE,
			snake_length: Self::SNAKE_LENGTH,
			snake_direction: Self::SNAKE_DIRECTION,
		}
	}
}

#[cfg(test)]
pub mod tests {
	use super::*;

	#[test]
	fn kill_dead_snakes() -> crate::Result<()> {
		let mut gd = GameData::new(Some((20, 20)), Default::default());

		gd.spawn_snake('1', Some((-1, -1).into()), None, Some(1))?;

		gd.spawn_snake('2', Some((15, 5).into()), None, Some(5))?;
		snake::bump_parts(gd.snake_mut('2')?)?;

		gd.spawn_snake('3', Some((4, 6).into()), None, Some(1))?;
		gd.spawn_snake('4', Some((3, 6).into()), None, Some(2))?;

		gd.kill_dead_snakes();

		assert!(!gd.find_snake('1'), "snake 1 should be dead");
		assert!(!gd.find_snake('2'), "snake 2 should be dead");
		assert!(!gd.find_snake('3'), "snake 3 should be dead");
		assert!(!gd.find_snake('4'), "snake 4 should be dead");

		Ok(())
	}
}
