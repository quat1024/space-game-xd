#![feature(array_windows)]
#![allow(dead_code)] //for now

use std::path::PathBuf;

use anyhow::*;
use asset_loader::AssetLoader;
use game::Game;
use render::GameRenderer;
use window::GameWindow;

mod asset_loader;
mod game;
mod render;
mod util;
mod window;
mod world;

fn main() -> Result<()> {
	env_logger::init();

	let mut asset_path: PathBuf = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| "./".to_string()).into();
	asset_path.push("assets");

	println!("asset base path: {:?}", asset_path);

	let asset_loader = AssetLoader::new(asset_path);
	let game = Game::load(&asset_loader)?;

	let window = GameWindow::new("my game name!", 1024, 576)?;
	let mut renderer = futures::executor::block_on(GameRenderer::new(&window, &asset_loader)).context("unable to create game renderer")?;

	//set up
	renderer.setup(&game);

	//go
	window.run_loop(game, renderer); //Never returns
	unreachable!()
}
