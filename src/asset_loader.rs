use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::*;
use wgpu::ShaderSource;

pub struct AssetLoader {
	base_path: PathBuf,
}

impl AssetLoader {
	pub fn new<T: Into<PathBuf>>(path: T) -> Self {
		AssetLoader {
		    base_path: path.into(),
		}
	}

	pub fn create_shader_module(&self, device: &wgpu::Device, name: &str) -> Result<wgpu::ShaderModule> {
		let mut path = self.base_path.clone();
		path.push("compiled_shaders");
		path.push(name);

		let contents = std::fs::read(path).with_context(|| format!("failed to load shader '{}'", name))?;

		Ok(device.create_shader_module(&wgpu::ShaderModuleDescriptor {
			label: Some(name),
			source: wgpu::util::make_spirv(&contents),
			flags: Default::default(),
		}))
	}
}
