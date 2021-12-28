#![allow(clippy::unused_io_amount)]

use clap::{App as CliApp, Arg};
use eframe::{
    egui::{self, epaint},
    epi,
};
use game::prelude::*;
use server::Client;
use std::net::TcpStream;

/// Print grid into stdout when available.
const DEBUG_GRID: bool = false;

fn main() {
    let matches = CliApp::new("Snake Game Client by Mark")
        .about("Allows connecting to some multiplayer server")
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
    let make_connection = matches.is_present("connect");

    let app = GuiApp::new(client_name, server_address, make_connection);
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(Box::new(app), native_options);
}

pub struct GuiApp {
    /// Client id.
    id: Option<String>,

    /// Initial client id.
    initial_id: Option<String>,

    /// Server address.
    address: Option<String>,

    /// Flag which determines does client needs to make a server connection.
    make_connection: bool,

    /// Server connection status.
    connection_status: String,

    /// Server stream.
    stream: Option<TcpStream>,

    /// Game grid.
    grid: Option<Grid>,
}

impl Client for GuiApp {
    fn set_stream(&mut self, stream: Option<TcpStream>) {
        self.stream = stream;
    }

    fn stream(&mut self) -> Option<&mut TcpStream> {
        self.stream.as_mut()
    }

    fn stream_clone(&self) -> Option<TcpStream> {
        self.stream
            .as_ref()
            .map(|stream| stream.try_clone().unwrap())
    }

    fn set_id(&mut self, id: Option<String>) {
        self.id = id
    }

    fn id(&self) -> Option<String> {
        self.id.clone()
    }
}

impl GuiApp {
    /// Return a new [`Client`]
    fn new(id: Option<String>, address: Option<String>, make_connection: bool) -> Self
where {
        Self {
            initial_id: id.clone(),
            id,
            address,
            make_connection,
            connection_status: String::new(),
            stream: None,
            grid: None,
        }
    }

    /// Connect to the server.
    ///
    /// # Panic
    /// Panics if `self.address` or `self.name` is none.
    fn connect(&mut self) {
        let address = self.address.clone().unwrap();
        self.make_connection = false;
        match <Self as Client>::connect(self, address) {
            Ok(_) => self.connection_status = String::from("Success"),
            Err(e) => self.connection_status = format!("Error: {}", e),
        }
    }

    /// Disconnect from the server.
    ///
    /// # Panic
    /// Panics if `self.stream` or `self.name` is None or if writing to the
    /// server buffer has failed.
    fn disconnect(&mut self) {
        self.make_connection = false;
        match <Self as Client>::disconnect(self) {
            Ok(_) => {
                self.stream = None;
                self.connection_status = String::from("Disconnected")
            }
            Err(e) => self.connection_status = format!("Error: {}", e),
        }
    }

    /// Reconnect to the server.
    fn reconnect(&mut self) {
        self.disconnect();
        self.connect();
    }
}

impl epi::App for GuiApp {
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
        if self.make_connection {
            self.connect();
        }

        if self.stream.is_none() {
            egui::Window::new("Connect to server").show(ctx, |ui| {
                let mut address = match self.address.clone() {
                    Some(val) => val,
                    None => String::new(),
                };
                let mut initial_id = match self.initial_id.clone() {
                    Some(val) => val,
                    None => String::new(),
                };

                ui.label("Server address:");
                ui.add(egui::TextEdit::singleline(&mut address));
                self.address = Some(address);

                ui.label("Player name:");
                ui.text_edit_singleline(&mut initial_id);

                self.initial_id = Some(initial_id.clone());
                self.id = Some(initial_id);

                if ui.button("Connect").clicked() || ctx.input().key_pressed(egui::Key::Enter) {
                    self.connection_status = String::from("Try connecting to server");
                    self.make_connection = true;
                };
                ui.label(self.connection_status.clone());
            });
        } else {
            self.grid = match self.request_grid() {
                Ok(grid) => Some(grid),
                Err(e) => {
                    self.connection_status = format!("Error while requesting a grid: {}", e);
                    self.make_connection = false;
                    self.stream = None;
                    return;
                }
            };

            egui::CentralPanel::default().show(ctx, |ui| {
                let grid = self.grid.clone().unwrap();

                if DEBUG_GRID {
                    println!(
                        "---\nDisplaying \"{}\" server's grid with {}x{} size:\n{}---\n",
                        self.address.clone().unwrap(),
                        grid.size.0,
                        grid.size.1,
                        grid
                    );
                }

                let cell = 20.0;
                let frame = cell; // frame stroke size
                let offset = cell * 2.0;

                let mut shapes: Vec<egui::Shape> = Vec::new();

                let grid = self.grid.clone().unwrap();

                shapes.push(egui::Shape::Rect(epaint::RectShape::stroke(
                    epaint::Rect {
                        min: egui::pos2(offset - frame, offset - frame),
                        max: egui::pos2(
                            (grid.size.0 as f32 * cell) + frame + cell * 2.0,
                            (grid.size.1 as f32 * cell) + frame + cell,
                        ),
                    },
                    0.0,
                    epaint::Stroke::new(frame, color32(Color::WHITE)),
                )));

                let offset = offset + frame / 2.0;

                for point in grid.data {
                    let (x, y) = (
                        point.coordinates.x as f32,
                        (grid.size.1 as i32 - point.coordinates.y) as f32,
                    );
                    shapes.push(egui::Shape::Rect(epaint::RectShape::filled(
                        epaint::Rect {
                            min: egui::pos2(cell * x + offset - cell, cell * y + offset - cell),
                            max: egui::pos2(cell * x + offset, cell * y + offset),
                        },
                        0.0,
                        color32(point.color),
                    )));
                }

                ui.painter().extend(shapes);
            });
            ctx.request_repaint();

            if ctx.input().key_pressed(egui::Key::W) {
                self.change_direction(Direction::Up).unwrap();
            } else if ctx.input().key_pressed(egui::Key::S) {
                self.change_direction(Direction::Down).unwrap();
            } else if ctx.input().key_pressed(egui::Key::A) {
                self.change_direction(Direction::Left).unwrap();
            } else if ctx.input().key_pressed(egui::Key::D) {
                self.change_direction(Direction::Right).unwrap();
            } else if ctx.input().key_pressed(egui::Key::R) {
                self.reconnect();
            } else if ctx.input().key_pressed(egui::Key::Escape) {
                self.disconnect();
            }

            egui::SidePanel::new(egui::panel::Side::Right, "disconnect_panel").show(ctx, |ui| {
                if ui.button("Disconnect").clicked() {
                    self.disconnect();
                };
            });
        }
    }

    fn on_exit(&mut self) {
        if self.stream.is_some() {
            self.disconnect();
        }
    }
}

fn color32(color: Color) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(color.r, color.g, color.b, color.a)
}
