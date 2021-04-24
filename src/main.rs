#![allow(unused_imports)]

use game::Game;
use render::GameRenderer;
use window::GameWindow;

use anyhow::*;

mod window;
mod game;
mod render;
mod asset_loader;

fn main() -> Result<()> {
    env_logger::init();
	println!("CARGO_MANIFEST_DIR: {}", std::env::var("CARGO_MANIFEST_DIR").unwrap());
	
	let window = GameWindow::new("my game name!", 1024, 576)?;
	let game = Game::new();
	let renderer = futures::executor::block_on(GameRenderer::new(&window)).context("unable to create game renderer")?;
	
	window.run_loop(game, renderer); //Never returns
	unreachable!()
}
