//! Game server module.
//!
//! # Communication between server and client
//! Server uses TCP/IP protocol for communications. When it's launched, it
//! starts waiting for requests from [`TcpStream`].
//!
//! Request is a binary json string with special format specified by request's
//! structure.
//!
//! When server gets raw request in binary json format, it converts it into
//! abstracted request structure and does something depending on its kind.
//!
//! After server does everything it was asked for, it generates response with a
//! request its [`Result`] contained. This response converts into raw
//! binary json and sends back into the [`stream`](TcpStream).
//!
//! # Specification for clients
//! How every client should communicate with server in short:
//! 1. Client writes connection request to server's stream
//! 2. Client reads server's stream for its name in json format
//! 3. Client sends requests to get game grid and reads server's stream for it
//! 4. Client may send requests to change snake direction
//! 5. Client sends disconnection request to server's stream
//!
//! ## Implementing own client on Rust
//! If you write your client on Rust, then get familiar with [`Client`] trait
//! and implement it for your client abstraction and use related methods to do
//! things presented above.
//!
//! ## Implementing own client on another language
//! If you write your client on another language or you want to implement it on
//! Rust without this library or you just want to get how it works, then read
//! this section to know how to send or read requests in a right way and work
//! with the server without any helpers.
//!
//! If you just want to create a client, you can read only **Request** section
//! below. But if you want to know almost everything about server working
//! algorithm, read further sections.
//!
//! ### Request
//! Request is a binary json data with a special format. Every request has a
//! kind (connect, disconnect, get grid and so on) and unique identifier of a
//! client which sends it.
//!
//! There should also be put four null bytes after every request to allow server
//! splitting many requests in a read.
//!
//! #### Request to connect
//! ```json
//! {
//!     "client": "client identifier",
//!     "kind": "connect"
//! }
//! ```
//! This request should be sent at first and only once to authorize a client.
//!
//! After this request client should read server's stream for json string
//! containing its accepted identifier. Server will send something like this:
//! ```json
//! "client identifier"
//! ```
//!
//! #### Request to get game grid
//! ```json
//! {
//!     "client": "client identifier",
//!     "kind": "get_grid"
//! }
//! ```
//! This request should be sent constantly to get game grid.
//!
//! ### Request to change snake's direction
//! ```json
//! {
//!     "client": "client identifier",
//!     "kind": {
//!         "change_direction": "right"
//!     }
//! }
//! ```
//! This request should be sent constantly to get game grid.
//! There "change_direction" can have "up", "down", "left" or "right" values.
//!
//! #### Request to connect
//! ```json
//! {
//!     "client": "client identifier",
//!     "kind": "disconnect"
//! }
//! ```
//! This request should be sent at last and only once to deauthorize the client.
//!
//! ### Response
//! Response is a result of processing a request.
//!
//! ### Exchange
//! Exchange is a request linked with its response. If there's no response
//! linked, then exchange is called *uncompleted*. Otherwise exchange is called
//! *completed*. Vector/Stack/Array/Pool of exchanges is named *exchange pool*.
//!
//! ### Session
//! When client connects to the server, a session is started. Session is a
//! structure which contains exchange pool and server options.

use crate::{
	game::{Direction, GameData, Grid},
	Result,
};
use serde::{Deserialize, Serialize};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::{
	error,
	fmt::{self, Debug},
	io::{Read, Write},
	sync::{Arc, Mutex},
	thread,
	time::Duration,
};

/// How many bytes client can read from a stream at a time.
const READ_LIMIT: usize = 1024 * 10;

/// Default delay between every server response.
pub const GAME_DELAY: Duration = Duration::from_millis(70);

/// Trait which should be implemented for client abstractions.
pub trait Client {
	/// Connect to the server with specified address. `client` is a name of the
	/// snake. Return stream and client name taken from server connection response.
	fn connect<A: ToSocketAddrs + Debug>(&mut self, address: A) -> Result<()> {
		match TcpStream::connect(&address) {
			Ok(stream) => {
				self.set_stream(Some(stream));
				Request::new(self.id().unwrap(), RequestKind::Connect)
					.write(self.stream().unwrap())
					.expect("writing to the server stream");

				self.read_client_id()?;

				Ok(())
			}
			Err(e) => Err(Box::new(e)),
		}
	}

	/// Parse client id after reading stream after connection request.
	///
	/// This function should be used to parse returned by server client's id
	/// value after connection request.
	fn read_client_id(&mut self) -> Result<()> {
		let mut buffer = [0; READ_LIMIT];
		self.stream().unwrap().read(&mut buffer).unwrap();

		let name = String::from_utf8_lossy(&buffer);
		let trim_pattern: &[_] = &[char::from(0), '"'];

		self.set_id(Some(name.trim_matches(trim_pattern).to_string()));

		Ok(())
	}

	/// Send request to get game grid to server's stream, read for it and return
	/// read value.
	fn request_grid(&mut self) -> Result<Grid> {
		let id = self.id().unwrap();
		let stream = self.stream().unwrap();

		Request::new(id, RequestKind::GetGrid).write(stream)?;

		let mut buffer = [0; READ_LIMIT];

		stream.read(&mut buffer)?;

		let string = String::from_utf8_lossy(&buffer);

		Grid::from_string(&string.trim_matches(char::from(0)))
	}

	/// Send request to disconnect from the server.
	fn disconnect(&mut self) -> Result<()> {
		let id = self.id().unwrap();
		let stream = self.stream().unwrap();

		Request::new(id, RequestKind::Disconnect).write(stream)?;

		stream.flush()?;

		Ok(())
	}

	/// Send request to change snake's direction.
	fn change_direction(&mut self, direction: Direction) -> Result<()> {
		Request::new(
			self.id().unwrap(),
			RequestKind::ChangeDirection(direction),
		)
		.write(self.stream().unwrap())?;
		Ok(())
	}

	/// Set client's stream.
	fn set_stream(&mut self, stream: Option<TcpStream>);

	/// Return mutable reference to [`server stream`](TcpStream).
	fn stream(&mut self) -> Option<&mut TcpStream>;

	/// Return cloned [`server stream`](TcpStream).
	fn stream_clone(&self) -> Option<TcpStream>;

	/// Set client's identifier.
	fn set_id(&mut self, id: Option<String>);

	/// Return client's identifier.
	fn id(&self) -> Option<String>;
}

/// Run server with specified address and [`GameData`].
/// `delay` is a delay between every response, it may be used to slow down the
/// game. If `delay` is none, `GAME_DELAY` value is used.
pub fn run<A: ToSocketAddrs>(
	address: A,
	gamedata: GameData,
	game_delay: Option<Duration>,
) -> Result<()> {
	let listener = TcpListener::bind(address)?;
	let gamedata = Arc::new(Mutex::new(gamedata));
	let game_delay = game_delay.map_or(GAME_DELAY, |d| d);

	loop {
		let (socket, address) = match listener.accept() {
			Ok(val) => val,
			Err(e) => {
				log::error!("Failed to accept incoming connection: {}", e);
				continue;
			}
		};
		let gamedata = gamedata.clone();
		thread::spawn(
			move || match handle_client(socket, gamedata, Some(game_delay)) {
				Ok(_) => log::info!("Successfully handled client {}", address),
				Err(e) => log::error!("Failed to handle client \"{}\": {}", address, e),
			},
		);
	}
}

/// Handle client connected to server.
/// `delay` is a delay between every request, it may be used to slow down the
/// game.
fn handle_client(
	stream: TcpStream,
	gamedata: Arc<Mutex<GameData>>,
	delay: Option<Duration>,
) -> Result<()> {
	let mut session = Session::new(stream, gamedata.clone(), delay);

	loop {
		if session.wait().is_err() {
			continue;
		}

		if let Err(e) = session.handle_requests() {
			log::debug!(
				"{:?} {} - discard handling",
				session.client().unwrap_or_default(),
				e
			);
			session.discard_exchanges();
		}

		if session.is_disconnected() {
			break;
		}
	}

	let mut gamedata = gamedata.lock().expect("acquiring gamedata mutex");

	if let Some(exchange) = session.exchanges().first() {
		let name = exchange.request().client;
		if gamedata.find_snake(&name) {
			gamedata.kill_snake(name)?;
		}
	}

	Ok(())
}

/// Struct which represents responses stack with some connection-handling data
/// and server stream.
struct Session {
	/// Server stream.
	stream: TcpStream,

	/// Client name.
	client: Option<String>,

	/// GameData.
	gamedata: Arc<Mutex<GameData>>,

	/// Is client connected to server or not.
	connected: bool,

	/// Delay between every request, it may be used to slow down the game.
	delay: Option<Duration>,

	/// `exchanges` is just a vector of server requests linked with responses.
	exchanges: Vec<Exchange>,
}

impl Session {
	/// Return a new empty [`Session`].
	fn new(stream: TcpStream, gamedata: Arc<Mutex<GameData>>, delay: Option<Duration>) -> Self {
		Self {
			stream,
			gamedata,
			client: None,
			connected: false,
			delay,
			exchanges: vec![],
		}
	}

	/// Wait for requests and push them to the stack.
	fn wait(&mut self) -> Result<()> {
		let mut buffer = [0; 1024];

		self.stream.read(&mut buffer)?;

		if String::from_utf8_lossy(&buffer).trim_matches(char::from(0)) == "" {
			return Err(Box::new(ServerError::EmptyRequestString));
		}

		match Request::from_bytes(&buffer) {
			Ok(requests) => {
				for request in requests {
					self.exchanges_mut().push(Exchange(request.clone(), None));
				}
			}
			Err(e) => {
				log::error!("Failed to convert request: {}", e);
				return Err(e);
			}
		};

		Ok(())
	}

	/// Handle all uncompleted requests.
	fn handle_requests(&mut self) -> Result<()> {
		let mut is_connection_request = false;
		let mut stream = self.stream.try_clone()?;
		let gamedata = self.gamedata.clone();
		let delay = self.delay;
		let last_direction = self
			.exchanges()
			.iter()
			.filter(|exchange| exchange.completed())
			.map(|exchange| exchange.request().kind)
			.filter(|kind| matches!(kind, RequestKind::ChangeDirection(_)))
			.last();

		if self.exchanges().is_empty() {
			return Ok(());
		}

		let first_request = self.exchanges().first().unwrap().request();
		if !self.connected() && first_request.kind != RequestKind::Connect {
			return Err(Box::new(ServerError::IsNotConnected));
		}

		for exchange in self.exchanges_mut() {
			if exchange.response().is_some() {
				continue;
			}

			let mut request = exchange.request();

			// Lazily acquire gamedata mutex to work with it on a fly without
			// boilerplate code.
			let gamedata = || gamedata.lock().expect("acquiring gamedata mutex");

			let response = match request.kind {
				RequestKind::Connect => {
					let rng = rand::thread_rng();
					let snake_length = 1; //rng.gen_range(5..=10);
					let snake_coords = gamedata().grid().random_coords(snake_length, Some(rng));
					let mut name = request.client;

					// Check whether there is already a snake with such name and
					// if yes, change it to uniquely-generated one.
					if gamedata().find_snake(name.clone()) {
						name.push_str(&format!(" ({})", gamedata().snakes()));
					}

					is_connection_request = true;
					request.client = name.clone();

					Response::new(
						request.clone(),
						gamedata().spawn_snake(
							name,
							snake_coords,
							Direction::Right,
							snake_length as u32,
						),
					)
				}
				RequestKind::ChangeDirection(direction) => {
					if let Some(RequestKind::ChangeDirection(last_request_direction)) =
						last_direction
					{
						if last_request_direction == direction {
							return Err(Box::new(ServerError::IndenticalRequests));
						}
					}

					let mut gamedata = gamedata();
					let snake = gamedata.snake_mut(request.client.clone());

					match snake {
						Ok(snake) => {
							Response::new(request.clone(), snake.change_direction(direction))
						}
						Err(_) => Response::new(request.clone(), snake.map(|_| ())),
					}
				}
				RequestKind::GetGrid => Response::new(request.clone(), Ok(())),
				RequestKind::Disconnect => Response::new(
					request.clone(),
					gamedata().kill_snake(request.client()).map(|_| ()),
				),
			};

			if request.kind != RequestKind::GetGrid {
				log::info!("{}", response);
			}

			exchange.assign_response(response);

			gamedata().kill_dead_snakes();
			gamedata().check_apples()?;
			gamedata().update_grid()?;

			if let Some(delay) = delay {
				thread::sleep(delay);
			}

			match request.kind {
				RequestKind::Connect => {
					let buffer = serde_json::to_string(&request.client())?;
					log::debug!("Writing name to stream: {}", buffer);
					stream.write(buffer.as_bytes())?;
				}
				RequestKind::GetGrid => {
					let buffer = match gamedata().grid().as_bytes() {
						Ok(val) => val,
						Err(e) => {
							log::error!("Failed to convert gamedata: {}", e);
							return Err(e);
						}
					};
					stream.write(&buffer)?;
				}
				RequestKind::Disconnect => break,
				_ => (),
			}
		}
		if !self.connected && is_connection_request {
			self.connected = true
		}
		Ok(())
	}

	/// Return true if some of sent requests have
	/// [`disconnect kind`](RequestKind::Disconnect).
	fn is_disconnected(&self) -> bool {
		for exchange in self.exchanges() {
			if let RequestKind::Disconnect = exchange.request().kind {
				return true;
			}
		}
		false
	}

	/// Return true if client is connected or false otherwise.
	fn connected(&self) -> bool {
		self.connected
	}

	/// Get immutable reference to session's exchange.
	fn exchanges(&self) -> &Vec<Exchange> {
		&self.exchanges
	}

	/// Get mutable reference to session's exchange.
	fn exchanges_mut(&mut self) -> &mut Vec<Exchange> {
		&mut self.exchanges
	}

	/// Return client name.
	fn client(&self) -> Option<String> {
		self.exchanges()
			.first()
			.map(|exchange| exchange.request().client())
	}

	/// Remove uncompleted exchanges from stack.
	pub fn discard_exchanges(&mut self) {
		for i in 0..self.exchanges().len() {
			if !self.exchanges_mut()[i].completed() {
				self.exchanges_mut().remove(i);
			}
		}
	}
}

/// Server request abstraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct Request {
	/// Client identifier.
	client: String,
	/// Kind of request to send.
	kind: RequestKind,
}

impl Request {
	/// Return a new [`Request`]
	fn new(client: impl Into<String>, kind: RequestKind) -> Self {
		Self {
			client: client.into(),
			kind,
		}
	}

	/// Convert [`Request`] to bytes.
	fn as_bytes(&self) -> Result<Vec<u8>> {
		Ok(self.to_string()?.as_bytes().to_vec())
	}

	/// Convert bytes to [`Vec<Request>`].
	fn from_bytes(b: &[u8]) -> Result<Vec<Self>> {
		let mut requests = vec![];
		let string = String::from_utf8_lossy(b);
		let string = string.trim_matches(char::from(0));
		let separator = &String::from_utf8_lossy(&[0; 4]).to_string();
		for slice in string.split(separator) {
			if !slice.is_empty() {
				requests.push(Self::from_string(slice)?);
			}
		}
		Ok(requests)
	}

	/// Convert [`Request`] to json string.
	fn to_string(&self) -> Result<String> {
		Ok(serde_json::to_string(self)?)
	}

	/// Convert json string to [`Request`].
	fn from_string<T: AsRef<str>>(string: T) -> Result<Self> {
		Ok(serde_json::from_str(
			string.as_ref().trim_matches(char::from(0)),
		)?)
	}

	/// Send request to server.
	///
	/// Write request to [`TcpStream`] after writing four null characters to
	/// make splitting multiple json requests possible.
	fn write(&self, stream: &mut TcpStream) -> Result<()> {
		stream.write(&self.as_bytes()?)?;
		stream.write(&[0; 4])?;
		Ok(())
	}

	/// Return client name.
	fn client(&self) -> String {
		self.client.clone()
	}
}

/// Enum of server request kinds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RequestKind {
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

/// Server response abstraction.
#[derive(Debug)]
struct Response {
	/// [`Request`] to answer.
	request: Request,

	/// Result of some game function.
	response: Result<()>,
}

impl Response {
	/// Return new [`Response`].
	fn new(request: Request, response: Result<()>) -> Self {
		Self { request, response }
	}

	/// Return [`Request`] linked to this response.
	fn request(&self) -> Request {
		self.request.clone()
	}
}

impl fmt::Display for Response {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self.response {
			Ok(_) => write!(
				f,
				"{}'s request to {} is successful",
				self.request.client, self.request.kind
			),
			Err(e) => write!(
				f,
				"{}'s request to {} is failed because {}",
				self.request.client, self.request.kind, e
			),
		}
	}
}

/// Struct representing request with possibly likned response.
#[derive(Debug)]
struct Exchange(Request, Option<Response>);

impl Exchange {
	/// Return linked request.
	fn request(&self) -> Request {
		self.0.clone()
	}

	/// Return possibly linked response.
	fn response(&self) -> &Option<Response> {
		&self.1
	}

	/// Assign response to the exchange.
	fn assign_response(&mut self, response: Response) {
		self.set_response(Some(response));
	}

	/// Unlink response from the exchange.
	fn unlink_response(&mut self) {
		self.set_response(None);
	}

	/// Set exchange response. Shouldn't be used directly, use
	/// [`assign_response`](Self::assign_response) or
	/// [`unlink_response`](Self::unlink_response) instead.
	fn set_response(&mut self, response: Option<Response>) {
		self.1 = response
	}

	/// Return true if there's a response assigned to that exchange.
	fn completed(&self) -> bool {
		self.1.is_some()
	}
}

/// Error type returned by [`server`](crate::server) module functions.
#[derive(Debug, Clone)]
pub enum ServerError {
	/// Client is trying to be handled without being authorized.
	///
	/// Every client should send connection request before being handled to be
	/// authorized by server.
	IsNotConnected,

	/// Client is sending nothing besides null characters.
	EmptyRequestString,

	/// Client sent two indentical requests. Requests to get some information
	/// are exceptions.
	IndenticalRequests,
}

impl fmt::Display for ServerError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::IsNotConnected => {
				write!(f, "client wants to be handled without being authorized")
			}
			Self::EmptyRequestString => write!(f, "client sent nothing besides null chars"),
			Self::IndenticalRequests => write!(f, "client sent two indentical requests"),
		}
	}
}

impl error::Error for ServerError {}
