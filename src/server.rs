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
use serde::{Deserialize, Serialize};
use std::{net::{TcpListener, TcpStream, ToSocketAddrs}, time::Duration};
use std::{
	fmt::{self, Debug},
	io::{Read, Write},
	thread,
};

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
pub fn run<A: ToSocketAddrs>(address: A, gamedata: GameData) -> Result<()> {
	let listener = TcpListener::bind(address)?;

	loop {
		let (socket, address) = match listener.accept() {
			Ok(val) => val,
			Err(e) => {
				eprintln!("Failed to accept incoming connection: {}", e);
				continue;
			}
		};
		let gamedata = gamedata.clone();
		thread::spawn(move || match handle_client(socket, gamedata) {
			Ok(_) => println!("Successfully handled client {}", address),
			Err(e) => eprintln!("Failed to handle client \"{}\": {}", address, e),
		});
	}
}

/// Handle client connected to server.
fn handle_client(mut stream: TcpStream, mut gamedata: GameData) -> Result<()> {
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
				RequestKind::Connect => Response::new(
					request.clone(),
					gamedata.spawn_snake(&request.clone().client, Direction::Right, 10),
				),
				RequestKind::ChangeDirection(direction) => {
					let snake = gamedata.snake(request.clone().client);
					match snake {
						Ok(snake) => Response::new(
							request.clone(),
							snake.change_direction(direction.clone()),
						),
						Err(_) => Response::new(request.clone(), snake.map(|_| ())),
					}
				}
				RequestKind::GetGameData => Response::new(request.clone(), Ok(())),
				RequestKind::Disconnect => Response::new(
					request.clone(),
					gamedata.kill_snake(request.client).map(|_| ()),
				),
			};

			println!("{}", response);

			thread::sleep(Duration::from_millis(300));
			gamedata.update_grid();

			if let RequestKind::Disconnect = request.kind {
				break 'a;
			}
		}

		let buffer = match gamedata.as_bytes() {
			Ok(val) => val,
			Err(e) => {
				eprintln!("Failed to convert gamedata: {}", e);
				return Err(e);
			}
		};

		stream.write(&buffer)?;
	}

	Ok(())
}

/// Enum of server request kinds.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestKind {
	/// Request to connect to server.
	Connect,
	/// Request to disconnect from server.
	Disconnect,
	/// Request to get game data.
	GetGameData,
	/// Request to change snake direction on the provided one.
	ChangeDirection(Direction),
}

impl fmt::Display for RequestKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Connect => write!(f, "connect to the server"),
			Self::Disconnect => write!(f, "disconnect from the server"),
			Self::GetGameData => write!(f, "get game data"),
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
