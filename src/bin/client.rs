#![allow(dead_code)]
#![allow(clippy::unused_io_amount)]

use clap::{App as CliApp, Arg};
use eframe::{
	egui::{self, epaint},
	epi::{self, App as GuiApp},
};
use snake_game::{
	game::{self, Grid},
	server,
};
use std::{io::Read, net::TcpStream};

fn main() {
	let matches = CliApp::new("Snake Game Client by Mark")
		.about("Lets connect to some multiplayer server")
		.arg(
			Arg::with_name("address")
				.short("a")
				.takes_value(true)
				.help("Server address"),
		)
		.arg(
			Arg::with_name("client_name")
				.short("n")
				.takes_value(true)
				.help("Snake name"),
		)
		.arg(Arg::with_name("connect").short("c").help(
			"Connect to server automatically if address and client name arguments are specified",
		))
		.get_matches();

	let server_address = matches.value_of("address").map(|val| val.to_string());
	let client_name = matches.value_of("client_name").map(|val| val.to_string());
	let connect = matches.is_present("connect");

	if connect {
		todo!();
	}

	let client = Client::new(client_name, server_address, connect);
	let native_options = eframe::NativeOptions::default();

	eframe::run_native(Box::new(client), native_options);
}

pub struct Client {
	/// Client name (snake name).
	name: Option<String>,

	/// Server address.
	address: Option<String>,

	/// Connect to server automatically if `name` and `address` fields
	/// specified.
	connect: bool,

	/// Server connection status.
	connection_status: String,

	/// Server stream.
	stream: Option<TcpStream>,

	/// Game grid which updates using GameData update_grid method.
	grid: Option<Grid>,
}

impl Client {
	/// Return a new [`Client`]
	fn new(name: Option<String>, address: Option<String>, connect: bool) -> Self
where {
		Self {
			name,
			address,
			connect,
			connection_status: String::new(),
			stream: None,
			grid: None,
		}
	}

	/// Return cloned [`TcpStream`].
	fn stream(&self) -> TcpStream {
		self.stream.as_ref().unwrap().try_clone().unwrap()
	}

	/// Request grid from the server. Should be ran only after sending
	/// connection request to the server.
	fn request_grid(&mut self) -> snake_game::Result<Grid> {
		//std::thread::sleep(std::time::Duration::from_millis(100));

		let mut buffer = [0; 1024 * 10];

		let mut stream = self.stream();

		server::Request::new(self.name.clone().unwrap(), server::RequestKind::GetGrid)
			.write(&mut stream)
			.unwrap();

		stream.read(&mut buffer)?;

		let string = String::from_utf8_lossy(&buffer);

		//std::thread::sleep(std::time::Duration::from_millis(100));

		game::Grid::from_string(&string.trim_matches(char::from(0)))
	}

	/// Disconnect from the server.
	///
	/// # Panic
	/// Panics if `self.stream` or `self.name` is None or if writing to the
	/// server buffer has failed.
	fn disconnect(&mut self) {
		let mut stream = self.stream();

		server::Request::new(self.name.clone().unwrap(), server::RequestKind::Disconnect)
			.write(&mut stream)
			.unwrap();

		self.stream = None;
		self.connection_status = String::from("Disconnected");
		self.connect = false;
	}
}

impl GuiApp for Client {
	fn name(&self) -> &str {
		"Snake Game by Mark"
	}

	fn setup(
		&mut self,
		ctx: &egui::CtxRef,
		_frame: &mut epi::Frame<'_>,
		_storage: Option<&dyn epi::Storage>,
	) {
		ctx.set_visuals(egui::Visuals::dark());
	}

	fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
		if !self.connect {
			egui::Window::new("Connect to server").show(ctx, |ui| {
				let mut address = match self.address.clone() {
					Some(val) => val,
					None => String::new(),
				};
				let mut name = match self.name.clone() {
					Some(val) => val,
					None => String::new(),
				};

				ui.label("Server address:");
				ui.add(egui::TextEdit::singleline(&mut address));
				self.address = Some(address);

				ui.label("Player name:");
				ui.text_edit_singleline(&mut name);
				self.name = Some(name);

				if ui.button("Connect").clicked() {
					self.connection_status = String::from("Try connecting to server");
					match server::connect(self.address.clone().unwrap(), self.name.clone().unwrap())
					{
						Ok(stream) => {
							self.connection_status = String::from("Success");
							self.stream = Some(stream);
							self.connect = true;
						}
						Err(e) => {
							self.connection_status = format!("Error: {}", e);
						}
					}
				};
				ui.label(self.connection_status.clone());
			});
		} else if self.stream.is_some() {
			self.grid = match self.request_grid() {
				Ok(grid) => Some(grid),
				Err(e) => {
					self.connection_status = format!("Error while requesting a grid: {}", e);
					self.connect = false;
					self.stream = None;
					return;
				}
			};

			egui::CentralPanel::default().show(ctx, |ui| {
				let grid = self.grid.clone().unwrap();

				println!(
					"---\nDisplaying \"{}\" server's grid with {}x{} size:\n{}---\n",
					self.address.clone().unwrap(),
					grid.size.0,
					grid.size.1,
					grid
				);

				let offset = 10.0;
				let cell = 10.0;
				let frame = 10.0; // frame stroke size
				let mut shapes: Vec<egui::Shape> = Vec::new();

				let grid = self.grid.clone().unwrap();

				shapes.push(egui::Shape::Rect(epaint::RectShape::stroke(
					epaint::Rect {
						min: egui::pos2(offset - frame, offset - frame),
						max: egui::pos2(
							(grid.size.0 as f32 * cell) + frame,
							(grid.size.1 as f32 * cell) + frame,
						),
					},
					0.0,
					epaint::Stroke::new(frame, color32(game::Color::WHITE)),
				)));

				let offset = offset + frame / 2.0;

				for point in grid.data {
					let (x, y) = (
						point.coordinates.x as f32,
						(grid.size.1 as i32 - point.coordinates.y) as f32,
					);
					shapes.push(egui::Shape::Rect(epaint::RectShape::filled(
						epaint::Rect {
							min: egui::pos2(
								cell * x + offset - cell,
								cell * y + offset - cell,
							),
							max: egui::pos2(cell * x + offset, cell * y + offset),
						},
						0.0,
						color32(point.color),
					)));
				}

				ui.painter().extend(shapes);
			});
			ctx.request_repaint();

			let mut stream = self.stream();

			if ctx.input().key_pressed(egui::Key::W) {
				server::Request::new(
					self.name.clone().unwrap(),
					server::RequestKind::ChangeDirection(game::Direction::Up),
				)
				.write(&mut stream)
				.unwrap();
			} else if ctx.input().key_pressed(egui::Key::S) {
				server::Request::new(
					self.name.clone().unwrap(),
					server::RequestKind::ChangeDirection(game::Direction::Down),
				)
				.write(&mut stream)
				.unwrap();
			} else if ctx.input().key_pressed(egui::Key::A) {
				server::Request::new(
					self.name.clone().unwrap(),
					server::RequestKind::ChangeDirection(game::Direction::Left),
				)
				.write(&mut stream)
				.unwrap();
			} else if ctx.input().key_pressed(egui::Key::D) {
				server::Request::new(
					self.name.clone().unwrap(),
					server::RequestKind::ChangeDirection(game::Direction::Right),
				)
				.write(&mut stream)
				.unwrap();
			}

			egui::SidePanel::new(egui::panel::Side::Right, "disconnect_panel").show(ctx, |ui| {
				if ui.button("Disconnect").clicked() {
					self.disconnect();
				};
			});
		} else {
			self.connect = false;
		}
	}

	fn on_exit(&mut self) {
		if self.stream.is_some() {
			self.disconnect();
		}
	}
}

fn color32(color: game::Color) -> egui::Color32 {
	egui::Color32::from_rgba_premultiplied(color.r, color.g, color.b, color.a)
}
