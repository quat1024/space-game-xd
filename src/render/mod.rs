use anyhow::*;
use wgpu::*;
use winit::dpi::PhysicalSize;

use crate::{asset_loader::AssetLoader, world::Polyline};
use crate::game::Game;
use crate::window::GameWindow;

use self::polyline_renderer::PolylineRenderer;

mod polyline_renderer;

pub struct GameRenderer {
	pub bits: GameRendererBits,
	pub polyline_renderer: PolylineRenderer,
}

impl GameRenderer {
	pub async fn new(game_window: &GameWindow, asset_loader: &AssetLoader) -> Result<GameRenderer> {
		let bits = GameRendererBits::new(game_window).await?;
		
		let polyline_renderer = PolylineRenderer::new(&bits, asset_loader)?;
		
		Ok(GameRenderer {
			bits, polyline_renderer
		})
	}
	
	pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
		self.bits.resize(new_size)
	}

	pub fn recreate_swap_chain(&mut self) {
		self.bits.recreate_swap_chain()
	}
	
	pub fn render(&mut self, _game: &mut Game) -> std::result::Result<(), SwapChainError> {
		let frame = self.bits.sc.get_current_frame()?.output;
		let mut encoder = self.bits.device.create_command_encoder(&CommandEncoderDescriptor { label: None });
		
		let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
		    label: Some("My Pass"),
		    color_attachments: &[RenderPassColorAttachmentDescriptor {
		        attachment: &frame.view,
		        resolve_target: None,
		        ops: Operations {
		            load: LoadOp::Clear(Color {
		                r: 1.0,
		                g: 0.5,
		                b: 0.1,
		                a: 1.0,
					}),
		            store: true,
				}
			}],
		    depth_stencil_attachment: None,
		});
		
		//allocates new vertex buffers every frame DONT DO THIS IN THE REAL GAME!!!! LOLLL
		use ultraviolet::Vec2;
		use ultraviolet::Vec3;
		let polyline = Polyline {
		    points: vec![
				Vec2::new(0.0, 0.1),
				Vec2::new(0.1, 0.3),
				Vec2::new(0.5, 0.6),
				Vec2::new(1.0, 0.0),
			],
		    color: Vec3::new(0.0, 0.6, 1.0),
		    thickness: 0.3,
		};
		
		let polyline2 = Polyline {
		    points: vec![
				Vec2::new(-0.0, 0.1),
				Vec2::new(-0.1, 0.3),
				Vec2::new(-0.1, -0.3),
				Vec2::new(-1.0, -3.0),
				Vec2::new(-0.5, 0.6),
			],
		    color: Vec3::new(0.2, 0.2, 0.3),
		    thickness: 0.2,
		};
		
		self.polyline_renderer.tesselate(&self.bits.device, &[polyline, polyline2]);
		self.polyline_renderer.render(&mut pass);
		
		drop(pass);
		self.bits.queue.submit(std::iter::once(encoder.finish()));
		
		Ok(())
	}
}

pub struct GameRendererBits {
	pub surface: Surface,
	pub device: Device,
	pub queue: Queue,
	pub size: PhysicalSize<u32>,
	pub sc_desc: SwapChainDescriptor,
	pub sc: SwapChain,
	//pub pipeline: RenderPipeline,
}

impl GameRendererBits {
	pub async fn new(game_window: &GameWindow) -> Result<GameRendererBits> {
		let size = game_window.window.inner_size();

		let instance = Instance::new(BackendBit::PRIMARY);
		let surface = unsafe { instance.create_surface(&game_window.window) };

		let adapter = instance
			.request_adapter(&RequestAdapterOptions { power_preference: PowerPreference::HighPerformance, compatible_surface: Some(&surface) })
			.await
			.ok_or_else(|| anyhow!("failed to create adapter"))?;

		let (device, queue) = adapter
			.request_device(&DeviceDescriptor { label: None, features: Default::default(), limits: Default::default() }, None)
			.await
			.context("failed to create device and queue")?;

		let sc_desc = SwapChainDescriptor {
			usage: TextureUsage::RENDER_ATTACHMENT,
			format: adapter.get_swap_chain_preferred_format(&surface),
			width: size.width,
			height: size.height,
			present_mode: PresentMode::Fifo,
		};

		let sc = device.create_swap_chain(&surface, &sc_desc);

		Ok(GameRendererBits { surface, device, queue, size, sc_desc, sc })
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
}
