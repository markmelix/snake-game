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
        "Request that client should send when it wants to get game grid:\n{}\n",
        Request::new(
            "Client name (can consist of any characters and be non-uniq)".to_string(),
            RequestKind::GetGrid,
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

    gd.spawn_snake("Snake1", gd.grid().random_coords(10), Direction::Right, 10)
        .unwrap();
    gd.spawn_snake("Snake2", gd.grid().random_coords(10), Direction::Left, 10)
        .unwrap();

    gd.update_grid();

    println!(
        "Recevied grid:\n{}\nNote that \"object_kind\" can be either \"snake_part\" or \"apple\"",
        serde_json::to_string_pretty(&gd.grid()).unwrap()
    );
}
