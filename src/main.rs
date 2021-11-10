#[macro_use]
extern crate lazy_static;
use ggez::{event, GameResult};
mod game;

const SCREEN_SIZE: (f32, f32) = (
    game::GRID_SIZE[0] as f32 * game::GRID_CELL_SIZE[0] as f32,
    game::GRID_SIZE[1] as f32 * game::GRID_CELL_SIZE[1] as f32,
);

fn main() -> GameResult {
    let (ctx, events_loop) = ggez::ContextBuilder::new("BlockBreaker", "Piotr Paturej")
        .window_setup(ggez::conf::WindowSetup::default().title("BlockBreaker"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()?;

    let state = game::GameState::new();
    event::run(ctx, events_loop, state)
}
