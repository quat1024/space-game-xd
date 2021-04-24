use anyhow::*;
use ultraviolet::Vec2;
use ultraviolet::Vec3;

pub struct World {
	lines: Vec<LineSegment>,
}

pub struct Polyline {
	pub points: Vec<Vec2>,
	pub color: Vec3,
	pub thickness: f32,
}

impl Polyline {
	pub fn new(points: Vec<Vec2>, color: Vec3, thickness: f32) -> Self {
		Polyline { points, color, thickness }
	}

	//optimization idea: if the line doesn't change, b - a doesn't change either

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

#[derive(Clone, PartialEq)]
pub struct LineSegment {
	start: Vec2,
	end: Vec2,
	thickness: f32,
}

impl LineSegment {
	//pub const LAYOUT: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![0 => Float2, 1 => Float2, 2 => Float];

	pub fn new(start: Vec2, end: Vec2, thickness: f32) -> Self {
		Self { start, end, thickness }
	}

	pub fn flip(&mut self) {
		std::mem::swap(&mut self.start, &mut self.end);
	}

	/// based on Inigo Quilez's SDF for line segments
	pub fn distance_to_point(&self, point: Vec2) -> f32 {
		let pa = point - self.start;
		let ba = self.end - self.start;
		let h = (pa.dot(ba) / ba.dot(ba)).clamp(0.0, 1.0);
		(pa - ba * h).mag()
	}

	/// based on Inigo Quilez's SDF for line segments
	pub fn contains(&self, point: Vec2) -> bool {
		let pa = point - self.start;
		let ba = self.end - self.start;
		let h = pa.dot(ba) / ba.dot(ba);
		if h > 0.0 && h < 1.0 {
			(pa - ba * h).mag_sq() <= self.thickness * self.thickness
		} else {
			false
		}
	}
}
