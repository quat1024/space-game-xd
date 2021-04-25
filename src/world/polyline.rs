use ultraviolet::Vec2;
use ultraviolet::Vec3;

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
