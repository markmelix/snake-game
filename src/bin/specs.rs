//! Generate specs
use snake_game::{game::*, server::*};

fn main() {
	println!(
		"First request client should send when it connects to the server:\n{}\n",
		Request::new(
			"Client name (can consist of any characters and be non-uniq)".to_string(),
			RequestKind::Connect,
		)
		.to_string()
		.unwrap()
	);

	println!(
		"Request that client should send when it disconnects from the server:\n{}\n",
		Request::new(
			"Client name (can consist of any characters and be non-uniq)".to_string(),
			RequestKind::Disconnect,
		)
		.to_string()
		.unwrap()
	);

	println!(
		"Request that client should send when player wants to change direction of the snake:\n{}\n",
		Request::new(
			"Client name (can consist of any characters and be non-uniq)".to_string(),
			RequestKind::ChangeDirection(Direction::Right),
		)
		.to_string()
		.unwrap()
	);

	let mut gd = GameData::default();

	gd.spawn_snake("Anton", Direction::Right, 10).unwrap();
	gd.spawn_snake("Mark", Direction::Left, 10).unwrap();

	gd.update_grid();

	println!("Recevied GameData:\n{}", serde_json::to_string_pretty(&gd.grid()).unwrap());
}
