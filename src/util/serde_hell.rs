use serde::Deserialize;
use serde::Serialize;

/// Copy of ultraviolet::Vec2
#[derive(Serialize, Deserialize)]
#[serde(remote = "ultraviolet::Vec2")]
pub struct NotVec2 {
	x: f32,
	y: f32,
}

/// This is ALLLL because serde's "with" attributes works for fields of type X but not Z<X> even if Z is serializable
pub mod vec_of_vec2 {
	use serde::de::SeqAccess;
	use serde::de::Visitor;
	use serde::ser::SerializeSeq;
	use serde::Deserializer;
	use serde::Serializer;
	use ultraviolet::Vec2;

	use super::*;

	pub fn serialize<S>(things: &[Vec2], s: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut y = s.serialize_seq(Some(things.len()))?;
		for thing in things {
			let yeet: AlsoNotVec2 = thing.into();
			y.serialize_element(&yeet)?;
		}
		y.end()
	}

	pub fn deserialize<'de, D>(d: D) -> Result<Vec<Vec2>, D::Error>
	where
		D: Deserializer<'de>,
	{
		d.deserialize_seq(Suffering { lkjdkjskdjskd: Vec::new() })
	}

	//Using "remote = whatever" makes serde not *actually* implement serialize for your struct so idk here's another one!!
	#[derive(Serialize, Deserialize)]
	pub struct AlsoNotVec2 {
		x: f32,
		y: f32,
	}

	impl AlsoNotVec2 {
		fn to_ultraviolet(&self) -> Vec2 {
			Vec2 { x: self.x, y: self.y }
		}
	}

	impl From<&Vec2> for AlsoNotVec2 {
		fn from(sd: &Vec2) -> Self {
			AlsoNotVec2 { x: sd.x, y: sd.y }
		}
	}

	struct Suffering {
		lkjdkjskdjskd: Vec<Vec2>,
	}

	impl<'de> Visitor<'de> for Suffering {
		type Value = Vec<Vec2>;

		fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
			formatter.write_str("a vec2")
		}

		fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
		where
			A: SeqAccess<'de>,
		{
			while let Some(alksdklas) = seq.next_element::<AlsoNotVec2>()? {
				self.lkjdkjskdjskd.push(alksdklas.to_ultraviolet());
			}

			Ok(self.lkjdkjskdjskd)
		}
	}
}

/// Copy of ultraviolet::Vec3
#[derive(Serialize, Deserialize)]
#[serde(remote = "ultraviolet::Vec3")]
pub struct NotVec3 {
	x: f32,
	y: f32,
	z: f32,
}
