#![allow(dead_code)]

const DEFAULT_ADDRESS: &str = "localhost:8787";
const DEFAULT_CLIENT_NAME: &str = "Snake";

use clap::{App, Arg};
use iced::{
	self,
	canvas::{Canvas, Cursor, Frame, Geometry, Path, Program},
	executor, keyboard, Application, Clipboard, Color, Command, Container, Element, Length,
	Rectangle, Settings, Size, Subscription,
};
use iced_native::{subscription, Event};
use snake_game::{
	game::{GameData, Grid},
	server,
};
use std::{fmt::Debug, sync::Arc};
use tokio::{
	io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
	net::TcpStream,
	sync::Mutex,
};

fn main() -> iced::Result {
	Client::run(Settings::default())
}

/// GUI client implementation.
#[derive(Debug)]
pub struct Client {
	/// Client name (snake name).
	client: String,

	/// Server streams.
	read_stream: Option<Arc<Mutex<ReadHalf<TcpStream>>>>,
	write_stream: Option<Arc<Mutex<WriteHalf<TcpStream>>>>,

	/// Game grid which updates using update_grid method.
	grid: Grid,
}

/// Request grid from the server.
async fn request_grid(stream: Arc<Mutex<ReadHalf<TcpStream>>>) -> Grid {
	let mut buffer = [0; 1024 * 10];
	stream
		.lock()
		.await
		.read(&mut buffer)
		.await
		.expect("reading stream buffer");
	let string = String::from_utf8_lossy(&buffer);
	println!("{:?}", string.trim_matches(char::from(0)));
	let gamedata = GameData::from_string(&string.trim_matches(char::from(0)))
		.expect("parsing json string to get gamedata");
	gamedata.grid()
}

#[derive(Debug)]
pub enum Message {
	UpPressed,
	DownPressed,
	LeftPressed,
	RightPressed,

	/// Changed snake direction.
	DirectionChanged,

	/// Connected to a server.
	Connected(TcpStream),

	GridRequested(Grid),
	RequestGrid(std::time::Instant),
}

impl Application for Client {
	type Executor = executor::Default;
	type Flags = ();
	type Message = Message;

	fn new(_flags: ()) -> (Self, Command<Message>) {
		let matches = App::new("Snake Game Client by Mark")
			.about("Lets connect to some multiplayer server")
			.arg(
				Arg::with_name("address")
					.default_value(DEFAULT_ADDRESS)
					.help(&format!("Server address. Default is {}", DEFAULT_ADDRESS)),
			)
			.arg(
				Arg::with_name("client_name")
					.takes_value(true)
					.default_value(DEFAULT_CLIENT_NAME)
					.help(&format!("Snake name. Default is {}", DEFAULT_CLIENT_NAME)),
			)
			.get_matches();

		let address = matches.value_of("address").unwrap_or(DEFAULT_ADDRESS);
		let address = String::from(address);
		let client = matches
			.value_of("client_name")
			.unwrap_or(DEFAULT_CLIENT_NAME);
		let client = String::from(client);

		(
			Self {
				client: client.clone(),
				write_stream: None,
				read_stream: None,
				grid: vec![],
			},
			Command::perform(server::connect(address, client), Message::Connected),
		)
	}

	fn title(&self) -> String {
		String::from("Snake Game by Mark")
	}

	fn update(
		&mut self,
		message: Self::Message,
		_clipboard: &mut Clipboard,
	) -> Command<Self::Message> {
		match message {
			Self::Message::Connected(stream) => {
				let (read_stream, write_stream) = tokio::io::split(stream);
				self.read_stream = Some(Arc::new(Mutex::new(read_stream)));
				self.write_stream = Some(Arc::new(Mutex::new(write_stream)));
				Command::none()
				// println!("Connected. Requesting grid");
				// Command::perform(request_grid(stream), Message::GridRequested)
			}
			Self::Message::RequestGrid(_) => Command::perform(
				request_grid(self.read_stream.unwrap()),
				Message::GridRequested,
			),
			Self::Message::GridRequested(grid) => {
				println!("Requesting grid - Successful");
				self.grid = grid;
				Command::none()
			}
			Self::Message::DirectionChanged => Command::none(),
			Self::Message::UpPressed => {
				println!("Up pressed");
				Command::none()
			}
			Self::Message::DownPressed => {
				println!("Down pressed");
				Command::none()
			}
			Self::Message::LeftPressed => {
				println!("Left pressed");
				Command::none()
			}
			Self::Message::RightPressed => {
				println!("Right pressed");
				Command::none()
			}
		}
	}

	fn subscription(&self) -> Subscription<Self::Message> {
		//Command::perform(server::connect(address, client), Message::Connected);
		let mut subscriptions = vec![];
		if self.write_stream.is_some() && self.read_stream.is_some() {
			subscriptions.push(
				iced_futures::time::every(std::time::Duration::from_millis(10))
					.map(Message::RequestGrid),
			);
		}
		subscriptions.push(subscription::events_with(|event, _status| match event {
			Event::Keyboard(keyboard_event) => match keyboard_event {
				keyboard::Event::KeyPressed {
					key_code: keyboard::KeyCode::W,
					modifiers: _,
				} => Some(Message::UpPressed),
				keyboard::Event::KeyPressed {
					key_code: keyboard::KeyCode::S,
					modifiers: _,
				} => Some(Message::DownPressed),
				keyboard::Event::KeyPressed {
					key_code: keyboard::KeyCode::A,
					modifiers: _,
				} => Some(Message::LeftPressed),
				keyboard::Event::KeyPressed {
					key_code: keyboard::KeyCode::D,
					modifiers: _,
				} => Some(Message::RightPressed),
				_ => None,
			},
			_ => None,
		}));
		Subscription::batch(subscriptions)
	}

	fn view(&mut self) -> Element<Self::Message> {
		let canvas = Canvas::new(self)
			.width(Length::Units(100))
			.height(Length::Units(100));

		Container::new(canvas)
			.width(Length::Fill)
			.height(Length::Fill)
			.padding(20)
			.center_x()
			.center_y()
			.into()
	}
}

impl Program<Message> for Client {
	fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
		let mut frame = Frame::new(bounds.size());
		frame.scale(10.0);
		for point in &self.grid {
			let coords = point.coordinates;
			let color = color(point.color);
			let figure = Path::rectangle(coords.to_f32().into(), Size::new(1.0, 1.0));
			frame.fill(&figure, color);
		}

		vec![frame.into_geometry()]
	}
}

fn color(color: snake_game::game::Color) -> Color {
	Color::new(color.r, color.g, color.b, color.a)
}
