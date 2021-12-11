//! Game server module.
//!
//! Clients written on Rust should use this module to be implemented.
//!
//! Clients written on other languages should generate binary json depending on
//! [`Request`] struct.

use crate::{
	game::{Direction, GameData},
	Result,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
	fmt::{self, Debug},
	io::{Read, Write},
	thread, sync::{Mutex, Arc}, time::Duration
};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

/// Default delay between every server response.
pub const GAME_DELAY: Duration = Duration::from_millis(50);

/// Connect to the server with specified address. `client` is a name of the
/// snake.
pub fn connect<A: ToSocketAddrs + Debug>(
	address: A,
	client: impl Into<String>,
) -> Result<TcpStream> {
	match TcpStream::connect(&address) {
		Ok(mut stream) => {
			Request::new(client.into(), RequestKind::Connect)
				.write(&mut stream)
				.expect("writing to the server stream");
			Ok(stream)
		}
		Err(e) => Err(Box::new(e)),
	}
}

/// Run server with specified address and [`GameData`].
/// `delay` is a delay between every response, it may be used to slow down the
/// game. If `delay` is none, `GAME_DELAY` value is used.
pub fn run<A: ToSocketAddrs>(address: A, gamedata: GameData, game_delay: Option<Duration>) -> Result<()> {
	let listener = TcpListener::bind(address)?;
	let gamedata = Arc::new(Mutex::new(gamedata));
	let game_delay = game_delay.map_or(GAME_DELAY, |d| d);

	loop {
		let (socket, address) = match listener.accept() {
			Ok(val) => val,
			Err(e) => {
				eprintln!("Failed to accept incoming connection: {}", e);
				continue;
			}
		};
		let gamedata = gamedata.clone();
		thread::spawn(move || match handle_client(socket, gamedata, game_delay) {
			Ok(_) => println!("Successfully handled client {}", address),
			Err(e) => eprintln!("Failed to handle client \"{}\": {}", address, e),
		});
	}
}

/// Handle client connected to server.
/// `delay` is a delay between every request, it may be used to slow down the
/// game.
fn handle_client(mut stream: TcpStream, gamedata: Arc<Mutex<GameData>>, delay: Duration) -> Result<()> {
	'a: loop {
		let mut buffer = [0; 1024];
		stream.read(&mut buffer)?;
		if String::from_utf8(buffer.to_vec())
			.unwrap()
			.trim_matches(char::from(0))
			!= ""
		{
			let request = match Request::from_bytes(&buffer) {
				Ok(val) => val,
				Err(e) => {
					eprintln!("Failed to convert request: {}", e);
					return Err(e);
				}
			};

			let response = match request.clone().kind {
				RequestKind::Connect => {
					let snake_coords = gamedata.lock().unwrap().grid().random_coords(0);
					println!("{:?}", snake_coords);
					Response::new(
						request.clone(),
						gamedata.lock().unwrap().spawn_snake(
							&request.clone().client,
							snake_coords,
							Direction::Right,
							rand::thread_rng().gen_range(5..=10),
						))
					}
				RequestKind::ChangeDirection(direction) => {
					let mut gamedata = gamedata.lock().unwrap();
					let snake = gamedata.snake(request.clone().client);
					match snake {
						Ok(snake) => Response::new(
							request.clone(),
							snake.change_direction(direction.clone()),
						),
						Err(_) => Response::new(request.clone(), snake.map(|_| ())),
					}
				}
				RequestKind::GetGrid => Response::new(request.clone(), Ok(())),
				RequestKind::Disconnect => Response::new(
					request.clone(),
					gamedata.lock().unwrap().kill_snake(request.client).map(|_| ()),
				),
			};

			if request.kind != RequestKind::GetGrid {
				println!("{}", response);
			}

			gamedata.lock().unwrap().kill_dead_snakes();
			gamedata.lock().unwrap().update_grid();

			thread::sleep(delay);

			match request.kind {
				RequestKind::Disconnect => break 'a,
				RequestKind::GetGrid => {
					let buffer = match gamedata.lock().unwrap().grid().as_bytes() {
						Ok(val) => val,
						Err(e) => {
							eprintln!("Failed to convert gamedata: {}", e);
							return Err(e);
						}
					};
					stream.write(&buffer)?;
				}
				_ => (),
			}
		}
	}
	Ok(())
}

/// Enum of server request kinds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestKind {
	/// Request to connect to server.
	Connect,
	/// Request to disconnect from server.
	Disconnect,
	/// Request to get game grid.
	GetGrid,
	/// Request to change snake direction on the provided one.
	ChangeDirection(Direction),
}

impl fmt::Display for RequestKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Connect => write!(f, "connect to the server"),
			Self::Disconnect => write!(f, "disconnect from the server"),
			Self::GetGrid => write!(f, "get game grid"),
			Self::ChangeDirection(direction) => {
				write!(f, "change snake direction to {}", direction)
			}
		}
	}
}

/// Server request abstraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Request {
	/// Client name.
	client: String,
	/// Kind of request to send.
	kind: RequestKind,
}

impl Request {
	/// Return new [`Request`]
	pub fn new(client: String, kind: RequestKind) -> Self {
		Self { client, kind }
	}

	/// Convert [`Request`] to bytes.
	pub fn as_bytes(&self) -> Vec<u8> {
		self.to_string().unwrap().as_bytes().to_vec()
	}

	/// Convert bytes to [`Request`].
	pub fn from_bytes(b: &[u8]) -> Result<Self> {
		Self::from_string(String::from_utf8_lossy(b))
	}

	/// Convert [`Request`] to json string.
	pub fn to_string(&self) -> Result<String> {
		Ok(serde_json::to_string_pretty(self)?)
	}

	/// Convert json string to [`Request`].
	pub fn from_string<T: AsRef<str>>(string: T) -> Result<Self> {
		Ok(serde_json::from_str(
			string.as_ref().trim_matches(char::from(0)),
		)?)
	}

	/// Send request to server.
	///
	/// Write request to [`TcpStream`]
	pub fn write(&self, stream: &mut TcpStream) -> Result<()> {
		stream.write(&self.as_bytes())?;
		Ok(())
	}
}

/// Server response abstraction.
struct Response<T> {
	/// [`Request`] to answer.
	request: Request,

	/// Result of some game function.
	response: Result<T>,
}

impl<T> Response<T> {
	/// Return new [`Response`].
	fn new(request: Request, response: Result<T>) -> Self {
		Self { request, response }
	}
}

impl<T> fmt::Display for Response<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self.response {
			Ok(_) => write!(
				f,
				"Request from \"{}\" client: \"{}\" is successful",
				self.request.client, self.request.kind
			),
			Err(e) => write!(
				f,
				"Request from \"{}\" client: \"{}\" is failed: {}",
				self.request.client, self.request.kind, e
			),
		}
	}
}
