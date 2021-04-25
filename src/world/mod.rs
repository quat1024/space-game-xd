use anyhow::*;
use ultraviolet::Vec2;
use ultraviolet::Vec3;

use crate::asset_loader::AssetLoader;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct World {
	pub lines: Vec<Polyline>,
}

impl World {
	pub fn load(asset_loader: &AssetLoader) -> Result<Self> {
		let world_file = asset_loader.load_string("world/map.ron")?;

		let world: World = ron::from_str(&world_file).with_context(|| anyhow!("couldnt parse world file"))?;

		Ok(world)
	}

	pub fn print_sample() {
		let polyline = Polyline {
			points: vec![Vec2::new(50.0, 50.0), Vec2::new(150.0, 150.0), Vec2::new(250.0, 50.0), Vec2::new(350.0, 250.0)],
			color: Vec3::new(0.0, 0.6, 1.0),
			thickness: 35.0,
		};

		let polyline2 = Polyline {
			points: vec![Vec2::new(100.0, 100.0), Vec2::new(200.0, 800.0), Vec2::new(800.0, 200.0), Vec2::new(123.0, 456.0), Vec2::new(0.0, 0.0)],
			color: Vec3::new(0.2, 0.2, 0.3),
			thickness: 80.0,
		};

		let world = World { lines: vec![polyline, polyline2] };

		println!("{}", ron::ser::to_string_pretty(&world, ron::ser::PrettyConfig::new()).expect("aldlsajd"));
	}
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Polyline {
	#[serde(with = "crate::util::vec_of_vec2")]
	pub points: Vec<Vec2>,
	#[serde(with = "crate::util::NotVec3")]
	pub color: Vec3,
	pub thickness: f32,
}

impl Polyline {
	pub fn new(points: Vec<Vec2>, color: Vec3, thickness: f32) -> Self {
		Polyline { points, color, thickness }
	}

	//optimization idea: if the line doesn't change, b - a doesn't change either
	//neither does ba.dot(ba)

	pub fn distance_to(&self, point: Vec2) -> f32 {
		self.points
			.array_windows()
			.map(|&[a, b]| {
				let pa = point - a;
				let ba = b - a;
				let h = (pa.dot(ba) / ba.dot(ba)).clamp(0.0, 1.0);
				(pa - ba * h).mag_sq()
			})
			.fold(f32::INFINITY, |a, b| a.min(b))
			.sqrt()
	}

	pub fn contains(&self, point: Vec2) -> bool {
		self.points.array_windows().any(|&[a, b]| {
			let pa = point - a;
			let ba = b - a;
			let h = pa.dot(ba) / ba.dot(ba);
			if h > 0.0 && h < 1.0 {
				(pa - ba * h).mag_sq() <= self.thickness * self.thickness
			} else {
				false
			}
		})
	}
}
