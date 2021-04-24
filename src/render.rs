use wgpu::*;
use winit::dpi::PhysicalSize;
use anyhow::*;

use crate::window::GameWindow;
use crate::asset_loader::AssetLoader;
use crate::game::Game;

pub struct GameRenderer {
	surface: Surface,
	device: Device,
	queue: Queue,
	size: PhysicalSize<u32>,
	sc_desc: SwapChainDescriptor,
	sc: SwapChain,
	//pipeline: RenderPipeline,
}

impl GameRenderer {
	pub async fn new(game_window: &GameWindow) -> Result<GameRenderer> {
		let size = game_window.window.inner_size();
		
		let instance = Instance::new(BackendBit::PRIMARY);
		let surface = unsafe { instance.create_surface(&game_window.window) };
		
		let adapter = instance.request_adapter(&RequestAdapterOptions {
		    power_preference: PowerPreference::HighPerformance,
		    compatible_surface: Some(&surface),
		}).await.ok_or_else(|| anyhow!("failed to create adapter"))?;
		
		let (device, queue) = adapter.request_device(&DeviceDescriptor {
		    label: None,
		    features: Default::default(),
		    limits: Default::default(),
		}, None).await.context("failed to create device and queue")?;
		
		let sc_desc = SwapChainDescriptor {
			usage: TextureUsage::RENDER_ATTACHMENT,
			format: adapter.get_swap_chain_preferred_format(&surface),
			width: size.width,
			height: size.height,
			present_mode: PresentMode::Fifo
		};
		
		let sc = device.create_swap_chain(&surface, &sc_desc);
		
		Ok(GameRenderer {
		    surface,
		    device,
		    queue,
		    size,
		    sc_desc,
		    sc,
		})
	}
	
	pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
		self.size = new_size;
		
		self.sc_desc.width = new_size.width;
		self.sc_desc.height = new_size.height;
		self.recreate_swap_chain();
	}
	
	pub fn recreate_swap_chain(&mut self) {
		if self.sc_desc.width != 0 && self.sc_desc.height != 0 {
			self.sc = self.device.create_swap_chain(&self.surface, &self.sc_desc);
		}
	}
	
	pub fn render(&mut self, _game: &mut Game) -> std::result::Result<(), SwapChainError> {
		Ok(())
	}
}