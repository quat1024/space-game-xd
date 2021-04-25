use anyhow::*;

use crate::asset_loader::AssetLoader;
use crate::world::Polyline;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GameWorld {
	pub lines: Vec<Polyline>,
}

impl GameWorld {
	pub fn load(asset_loader: &AssetLoader) -> Result<Self> {
		let world_file = asset_loader.load_string("world/map.ron")?;

		let world: GameWorld = ron::from_str(&world_file).with_context(|| anyhow!("couldnt parse world file"))?;

		Ok(world)
	}
}
