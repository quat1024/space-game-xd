use anyhow::*;
use util::DeviceExt;
use wgpu::*;
use winit::dpi::PhysicalSize;

use self::polyline_renderer::PolylineRenderer;
use crate::asset_loader::AssetLoader;
use crate::game::Game;
use crate::window::GameWindow;
use crate::world::Polyline;

mod polyline_renderer;

pub struct GameRenderer {
	pub bits: GameRendererBits,
	pub polyline_renderer: PolylineRenderer,
}

impl GameRenderer {
	pub async fn new(game_window: &GameWindow, asset_loader: &AssetLoader) -> Result<GameRenderer> {
		let bits = GameRendererBits::new(game_window).await?;

		let polyline_renderer = PolylineRenderer::new(&bits, asset_loader)?;

		Ok(GameRenderer { bits, polyline_renderer })
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

		//Dont do this every frame in the real game, tessellation takes a (relative) ton of cpu power
		use ultraviolet::Vec2;
		use ultraviolet::Vec3;
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

		self.polyline_renderer.tessellate(&self.bits.queue, &[polyline2, polyline]);
		//////////////////////////////////////

		//write uniforms (doesn't reallllly need to happen every frame, practically speaking it will, no harm)
		self.bits.queue.write_buffer(&self.bits.uniform_buffer, 0, bytemuck::cast_slice(&[self.bits.uniforms]));

		let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
			label: Some("pass!"),
			color_attachments: &[RenderPassColorAttachmentDescriptor {
				attachment: &frame.view,
				resolve_target: None,
				ops: Operations { load: LoadOp::Clear(Color { r: 1.0, g: 0.5, b: 0.1, a: 1.0 }), store: true },
			}],
			depth_stencil_attachment: None,
		});
		
		// apply global uniforms
		pass.set_bind_group(0, &self.bits.uniform_bind_group, &[]);

		//render the scene
		self.polyline_renderer.render(&mut pass);

		//all done. submit to the gpu
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
	// for "global"ish uniforms, such as camera position
	pub uniforms: Uniforms,
	pub uniform_buffer: Buffer,
	pub uniform_bind_group_layout: BindGroupLayout,
	pub uniform_bind_group: BindGroup,
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

		let uniforms = Uniforms::new(size);

		let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Rendererbits uniform buffer"),
			contents: bytemuck::cast_slice(&[uniforms]),
			usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
		});

		let uniform_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
			label: Some("Rendererbits uniform bind layout"),
			entries: &[BindGroupLayoutEntry {
				binding: 0,
				visibility: ShaderStage::VERTEX,
				ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
				count: None,
			}],
		});

		let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
			label: Some("Rendererbits uniform binds"),
			layout: &uniform_bind_group_layout,
			entries: &[BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() }],
		});

		Ok(GameRendererBits { surface, device, queue, size, sc_desc, sc, uniforms, uniform_buffer, uniform_bind_group_layout, uniform_bind_group })
	}

	pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
		self.size = new_size;

		self.sc_desc.width = new_size.width;
		self.sc_desc.height = new_size.height;
		self.recreate_swap_chain();

		self.uniforms.update(new_size);
	}

	pub fn recreate_swap_chain(&mut self) {
		if self.sc_desc.width != 0 && self.sc_desc.height != 0 {
			self.sc = self.device.create_swap_chain(&self.surface, &self.sc_desc);
		}
	}
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
	pub pixel_to_ndc: [[f32; 4]; 4],
}

impl Uniforms {
	fn new(size: PhysicalSize<u32>) -> Self {
		use ultraviolet::projection::*;

		//Left, right, bottom, top, near, far
		let mat = orthographic_wgpu_dx(0.0, size.width as f32, size.height as f32, 0.0, -1.0, 1.0);
		Self { pixel_to_ndc: mat.into() }
	}

	fn update(&mut self, size: PhysicalSize<u32>) {
		self.pixel_to_ndc = ultraviolet::projection::orthographic_wgpu_dx(0.0, size.width as f32, size.height as f32, 0.0, -1.0, 1.0).into();
	}
}
