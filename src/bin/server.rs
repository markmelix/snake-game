const DEFAULT_PORT: &str = "8787";

use clap::{App, Arg};
use snake_game::{game::GameData, server};

#[tokio::main]
async fn main() {
	let matches = App::new("Snake Game by Mark")
		.about("Lets start own multiplayer server")
		.arg(
			Arg::with_name("port")
				.short("p")
				.long("port")
				.help(&format!("Server port. Default is {}", DEFAULT_PORT)),
		)
		.arg(
			Arg::with_name("grid_size")
				.short("-g")
				.long("grid-size")
				.value_name("SIZE")
				.help(&format!(
					"Specifies game grid size. Default is {}x{}",
					GameData::DEFAULT_GRID_SIZE.0,
					GameData::DEFAULT_GRID_SIZE.1
				)),
		)
		.arg(
			Arg::with_name("snakes")
				.short("-s")
				.long("snakes")
				.value_name("NUMBER")
				.help(&format!(
					"Specifies maximum amount of snakes on the server. Default is {}",
					GameData::RECOMMENDED_SNAKES_AMOUNT
				)),
		)
		.arg(
			Arg::with_name("apples")
				.short("-a")
				.long("apples")
				.help(&format!(
					"Specifies maximum amount of apples that can be spawned on the server. Default is {}",
						GameData::RECOMMENDED_APPLES_AMOUNT)),
		)
		.get_matches();

	let port = matches.value_of("port").unwrap_or(DEFAULT_PORT);
	let grid_size: (usize, usize) = match matches.value_of("grid_size") {
		Some(val) => {
			let mut split = val
				.split('x')
				.map(|x| x.parse::<usize>().expect("Parsing grid size argument"));
			(split.next().unwrap(), split.next().unwrap())
		}
		None => GameData::DEFAULT_GRID_SIZE,
	};
	let snakes = match matches.value_of("snakes") {
		Some(val) => val.parse::<usize>().expect("Parsing snakes argument"),
		None => GameData::RECOMMENDED_SNAKES_AMOUNT,
	};
	let apples = match matches.value_of("apples") {
		Some(val) => val.parse::<usize>().expect("Parsing apples argument"),
		None => GameData::RECOMMENDED_APPLES_AMOUNT,
	};

	if let Err(e) = server::run(
		format!("localhost:{}", port),
		GameData::new(grid_size, Some(snakes), Some(apples)),
	)
	.await
	{
		eprintln!("Error while running the server: {}", e);
		return;
	}
}
