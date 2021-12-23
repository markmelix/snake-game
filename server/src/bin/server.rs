const DEFAULT_PORT: &str = "8787";

use std::time::Duration;

use snake_game::{game::GameData, server};

fn main() {
	init_logger();
	let settings = init_settings(init_cli());

	let address = format!("0.0.0.0:{}", settings.port);

	log::info!("Running server on {} address", address);

	if let Err(e) = server::run(
		address,
		GameData::new(
			Some(settings.grid_size),
			Some(settings.snakes),
			Some(settings.apples),
		),
		Some(settings.game_delay),
	) {
		log::error!("Error while running the server: {}", e);
		return;
	}
}

fn init_cli() -> clap::ArgMatches<'static> {
	use clap::{App, Arg};

	App::new("Snake Game by Mark")
		.about("Allows running own multiplayer server")
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
				.value_name("NUMBER")
				.help(&format!(
					"Specifies maximum amount of apples that can be spawned on the server. Default is {}",
						GameData::RECOMMENDED_APPLES_AMOUNT)),
		)
		.arg(
			Arg::with_name("game_delay")
				.short("-d")
				.long("delay")
				.value_name("DURATION")
				.help(&format!(
					"Specifies delay between every server response. Default is {:?}",
					server::GAME_DELAY
				)),
		)
		.get_matches()
}

struct Settings {
	port: String,
	grid_size: (usize, usize),
	snakes: usize,
	apples: usize,
	game_delay: Duration,
}

fn init_settings(matches: clap::ArgMatches) -> Settings {
	Settings {
		port: matches.value_of("port").unwrap_or(DEFAULT_PORT).to_string(),
		grid_size: match matches.value_of("grid_size") {
			Some(val) => {
				let mut split = val
					.split('x')
					.map(|x| x.parse::<usize>().expect("Parsing grid size argument"));
				(split.next().unwrap(), split.next().unwrap())
			}
			None => GameData::DEFAULT_GRID_SIZE,
		},
		snakes: match matches.value_of("snakes") {
			Some(val) => val.parse::<usize>().expect("Parsing snakes argument"),
			None => GameData::RECOMMENDED_SNAKES_AMOUNT,
		},
		apples: match matches.value_of("apples") {
			Some(val) => val.parse::<usize>().expect("Parsing apples argument"),
			None => GameData::RECOMMENDED_APPLES_AMOUNT,
		},
		game_delay: match matches.value_of("game_delay") {
			Some(val) => val
				.parse::<humantime::Duration>()
				.expect("Parsing delay argument")
				.into(),
			None => server::GAME_DELAY,
		},
	}
}

fn init_logger() {
	let log_level = match cfg!(debug_assertions) {
		true => "trace",
		false => "info",
	};

	env_logger::Builder::from_env(
		env_logger::Env::default()
			.filter_or("LOG_LEVEL", log_level)
			.write_style_or("LOG_STYLE", "auto"),
	)
	.format(|buf, record| {
		use env_logger::fmt::Color;
		use log::Level;
		use std::io::Write;

		let mut error = buf.style();
		let mut warn = buf.style();
		let mut info = buf.style();
		let mut debug = buf.style();
		let mut trace = buf.style();

		error.set_color(Color::Red).set_bold(true);
		warn.set_color(Color::Yellow);
		info.set_color(Color::Cyan);
		debug.set_color(Color::Magenta);
		trace.set_color(Color::Blue);

		let level_style = match record.level() {
			Level::Error => error,
			Level::Warn => warn,
			Level::Info => info,
			Level::Debug => debug,
			Level::Trace => trace,
		};

		writeln!(
			buf,
			"{}\t{}",
			level_style.value(record.level()),
			record.args()
		)
	})
	.init();
}
