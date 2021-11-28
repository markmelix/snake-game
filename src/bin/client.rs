#![allow(dead_code)]
#![allow(clippy::unused_io_amount)]

use clap::{App, Arg};
use eframe::{
	egui::{self, epaint, pos2, vec2, Vec2},
	epi,
};
use rand::Rng;
use snake_game::{
	game::{GameData, Grid},
	server,
};
use std::{io::Read, net::TcpStream, thread, time::Duration};

fn main() {
	let matches =
		App::new("Snake Game Client by Mark")
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
				"Connect to server automatically if address and client name arguments specified",
			))
			.get_matches();

	let server_address = matches.value_of("address").map(|val| val.to_string());
	let client_name = matches.value_of("client_name").map(|val| val.to_string());
	let connect = matches.is_present("connect");

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

	/// Request grid from the server. Should be ran only after sending
	/// connection request to the server.
	fn request_grid(&mut self) -> Grid {
		let mut buffer = [0; 1024 * 10];

		let mut stream = self.stream.as_ref().unwrap().try_clone().unwrap();

		server::Request::new(self.name.clone().unwrap(), server::RequestKind::GetGameData)
			.write(&mut stream)
			.unwrap();
		stream.read(&mut buffer).expect("reading stream buffer");

		let string = String::from_utf8_lossy(&buffer);

		let gamedata = GameData::from_string(&string.trim_matches(char::from(0)))
			.expect("parsing json string to get gamedata");

		gamedata.grid()
	}
}

impl epi::App for Client {
	fn name(&self) -> &str {
		"Snake Game by Mark"
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
							//thread::sleep(Duration::from_secs(1));
						}
						Err(e) => {
							self.connection_status = format!("Error: {}", e);
						}
					}
				};
				ui.label(self.connection_status.clone());
			});
		} else if self.stream.is_some() {
			self.grid = Some(self.request_grid());
			egui::CentralPanel::default().show(ctx, |ui| {
				let grid = self.grid.clone().unwrap();
				for point in grid.data {
					let color = color32(point.color);
					let image =
						egui::Image::new(egui::TextureId::User(u64::MAX), vec2(100.0, 100.0))
							.bg_fill(color)
							.tint(color);
					image.paint_at(
						ui,
						egui::Rect {
							min: pos2(point.coordinates.x as f32, point.coordinates.y as f32),
							max: pos2(
								(point.coordinates.x + 1) as f32 * 5.0,
								(point.coordinates.y - 1) as f32 * 5.0,
							),
						},
					);
				}
			});
			ctx.request_repaint();
		} else {
			self.connect = false;
		}
	}
}

fn color32(color: snake_game::game::Color) -> egui::Color32 {
	egui::Color32::from_rgb(
		color.r as u8,
		color.g as u8,
		color.b as u8,
		//color.a as u8,
	)
}
