use anyhow::*;

use crate::asset_loader::AssetLoader;
use crate::world::GameWorld;

pub struct Game {
	pub world: GameWorld,
}

impl Game {
	pub fn load(asset_loader: &AssetLoader) -> Result<Self> {
		Ok(Game { world: GameWorld::load(asset_loader)? })
	}

	pub fn handle_input(&mut self, _event: &winit::event::WindowEvent) -> bool {
		false
	}

	pub fn update(&mut self) {
		//Do nothing
	}
}
