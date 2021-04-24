use std::{collections::HashMap, path::PathBuf};
use anyhow::*;
use wgpu::ShaderSource;

pub struct AssetLoader {
	base_path: PathBuf,
}

impl AssetLoader {
	pub fn new(base_path: PathBuf) -> AssetLoader {
		AssetLoader {
			base_path,
		}
	}
	
	pub fn create_shader_module(&self, device: &wgpu::Device, name: &str) -> Result<wgpu::ShaderModule> {
		let mut path = self.base_path.clone();
		path.push("compiled_shaders");
		path.push(name);
		
		let contents = std::fs::read(path).map_err(|e| anyhow!("failed to read shader {} due to {}", name, e))?;
		
		Ok(device.create_shader_module(&wgpu::ShaderModuleDescriptor {
		    label: Some(name),
		    source: wgpu::util::make_spirv(&contents),
		    flags: Default::default(),
		}))
	}
}