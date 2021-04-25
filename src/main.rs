#![feature(array_windows)]
#![allow(unused_imports)] //for now
#![allow(dead_code)] //for now

use anyhow::*;
use asset_loader::AssetLoader;
use game::Game;
use render::GameRenderer;
use window::GameWindow;
use std::path::PathBuf;

mod asset_loader;
mod game;
mod render;
mod window;
mod world;

fn main() -> Result<()> {
	env_logger::init();
	
	let mut asset_path: PathBuf = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| "./".to_string()).into();
	asset_path.push("assets");
	
	println!("asset base path: {:?}", asset_path);

	let window = GameWindow::new("my game name!", 1024, 576)?;
	let game = Game::new();
	let asset_loader = AssetLoader::new(asset_path);
	let renderer = futures::executor::block_on(GameRenderer::new(&window, &asset_loader)).context("unable to create game renderer")?;

	window.run_loop(game, renderer); //Never returns
	unreachable!()
}
