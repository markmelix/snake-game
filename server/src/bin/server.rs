const DEFAULT_PORT: &str = "8787";

use std::time::Duration;

use game::prelude::*;
use logger::*;

fn main() {
    init_logger();
    let (port, grid_size, game_delay, settings) = init_settings(init_cli());

    let address = format!("0.0.0.0:{}", port);

    info!("Running server on {} address", address);

    if let Err(e) = server::run(
        address,
        GameData::new(Some(grid_size), settings),
        Some(game_delay),
    ) {
        error!("Error while running the server: {}", e);
    }
}

fn init_cli() -> clap::ArgMatches<'static> {
    use clap::{App, Arg};

    App::new("Snake Game by Mark")
        .about("Allows running own multiplayer server")
        .arg(
            Arg::with_name("port")
                .value_name("NUMBER")
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
                    GameData::GRID_SIZE.0,
                    GameData::GRID_SIZE.1
                )),
        )
        .arg(
            Arg::with_name("snakes")
                .short("-s")
                .long("snakes")
                .value_name("NUMBER")
                .help(&format!(
                    "Specifies maximum amount of snakes on the server. Default is {}",
                    Settings::SNAKES_AMOUNT
                )),
        )
        .arg(
            Arg::with_name("apples")
                .short("-a")
                .long("apples")
                .value_name("NUMBER")
                .help(&format!(
					"Specifies maximum amount of apples that can be spawned on the server. Default is {}",
						Settings::APPLES_AMOUNT)),
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
        .arg(
            Arg::with_name("snake_inc_size")
                .short("-i")
                .long("inc-size")
                .value_name("NUMBER")
                .help(&format!(
                    "Specifies snake increment size when it eats an apple. Default is {:?}",
                    Settings::SNAKE_INCREMENT_SIZE
                )),
        )
        .arg(
            Arg::with_name("snake_length")
                .short("l")
                .long("snake-length")
                .value_name("NUMBER")
                .help(&format!(
                    r"Specifies initial snake length on spawn. Note that you can either specify just
number N (for example: 1, 5, 15) or you can specify range M..(=)N(for example:
0..10, 5..15, 3..=5) to be used for generating initial snake length randomly
from M inclusively to N exclusively or if there is = before N, then N included
into the range. Default is {}",
                    Settings::SNAKE_LENGTH,
                )),
        )
        .arg(
            Arg::with_name("snake_step")
                .short("t")
                .long("snake-step")
                .value_name("NUMBER")
                .help(&format!(
                    "Specifies snake step. Default is {:?}",
                    Settings::SNAKE_STEP,
                )),
        )
        .arg(
            Arg::with_name("snake_direction")
                .short("r")
                .long("snake-direction")
                .value_name("DIRECTION")
                .help(&format!(
					"Specifies initial snake direction. Can be: left, right, up, down, random. Default is {:?}",
					match Settings::SNAKE_DIRECTION {
						Some(val) => format!("{}", val),
						None => "random".into(),
					},
				)),
        )
        .get_matches()
}

/// Initialize all server settings. Return server port, grid size, game delay
/// and game settings structure.
fn init_settings(matches: clap::ArgMatches) -> (String, (usize, usize), Duration, Settings) {
    (
        matches.value_of("port").unwrap_or(DEFAULT_PORT).to_string(),
        match matches.value_of("grid_size") {
            Some(val) => {
                let mut split = val
                    .split('x')
                    .map(|x| x.parse::<usize>().expect("Parsing grid size argument"));
                (
                    split
                        .next()
                        .expect("There should be two values separated with 'x'"),
                    split
                        .next()
                        .expect("There should be two values separated with 'x'"),
                )
            }
            None => GameData::GRID_SIZE,
        },
        match matches.value_of("game_delay") {
            Some(val) => val
                .parse::<humantime::Duration>()
                .expect("Parsing delay argument")
                .into(),
            None => server::GAME_DELAY,
        },
        Settings {
            snakes_amount: match matches.value_of("snakes") {
                Some(val) => val.parse::<usize>().expect("Parsing snakes argument"),
                None => Settings::SNAKES_AMOUNT,
            },
            apples_amount: match matches.value_of("apples") {
                Some(val) => val.parse::<usize>().expect("Parsing apples argument"),
                None => Settings::APPLES_AMOUNT,
            },
            snake_increment_size: match matches.value_of("snake_inc_size") {
                Some(val) => val
                    .parse::<usize>()
                    .expect("Parsing snake increment size argument"),
                None => Settings::SNAKE_INCREMENT_SIZE,
            },
            snake_length: match matches.value_of("snake_length") {
                Some(val) => val
                    .parse::<SnakeLength>()
                    .expect("Parsing snake length argument"),
                None => Settings::SNAKE_LENGTH,
            },
            snake_step: match matches.value_of("snake_step") {
                Some(val) => val.parse::<i32>().expect("Parsing snake step argument"),
                None => Settings::SNAKE_STEP,
            },
            snake_direction: match matches.value_of("snake_direction") {
                Some(val) => match val.parse::<Direction>() {
                    Ok(direction) => Some(direction),
                    Err(_) => {
                        if val == "random" {
                            None
                        } else {
                            panic!("Parsing snake direction argument")
                        }
                    }
                },
                None => Settings::SNAKE_DIRECTION,
            },
        },
    )
}
