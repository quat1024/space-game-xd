use anyhow::*;
use util::DeviceExt;
use wgpu::*;

use super::GameRendererBits;
use crate::asset_loader::AssetLoader;
use crate::world::Polyline;

/// A thingie that helps you render lines by tesselatting them into triangles.
pub struct PolylineRenderer {
	pipeline: RenderPipeline,
}

impl PolylineRenderer {
	#[allow(dead_code)] //no, it's used, r-a
	pub const BUFFER_SIZE: usize = 8192; //idk, how big can you make vertex buffers????

	pub fn new(game_renderer: &GameRendererBits, asset_loader: &AssetLoader) -> Result<Self> {
		let device = &game_renderer.device;

		let vert_module = asset_loader.create_shader_module(&device, "line.vert.spv").context("failed to load line renderer vert shader")?;
		let frag_module = asset_loader.create_shader_module(&device, "line.frag.spv").context("failed to load line renderer frag shader")?;

		let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
			label: Some("Line pipeline layout"),
			bind_group_layouts: &[&game_renderer.uniform_bind_group_layout],
			push_constant_ranges: &[],
		});

		let buffer_layout = VertexBufferLayout {
			array_stride: std::mem::size_of::<Vert>() as wgpu::BufferAddress,
			step_mode: InputStepMode::Vertex,
			attributes: &Vert::LAYOUT,
		};

		let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
			label: Some("Line pipeline"),
			layout: Some(&pipeline_layout),
			vertex: VertexState { module: &vert_module, entry_point: "main", buffers: &[buffer_layout] },
			fragment: Some(FragmentState { module: &frag_module, entry_point: "main", targets: &[game_renderer.sc_desc.format.into()] }),
			primitive: PrimitiveState {
				cull_mode: CullMode::None, //For now, until i get things debugged
				..Default::default()
			},
			depth_stencil: None,
			multisample: Default::default(),
		});

		Ok(PolylineRenderer { pipeline })
	}

	pub fn make_buffers(&self, device: &Device) -> PolylineBuffer {
		//Do you need to zero out buffers? Idk seems like a good idea
		let buncha_zeroes = vec![0; Self::BUFFER_SIZE];

		let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Line vertex buffer"),
			contents: bytemuck::cast_slice(&buncha_zeroes),
			usage: BufferUsage::COPY_DST | BufferUsage::VERTEX,
		});

		let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Line index buffer"),
			contents: bytemuck::cast_slice(&buncha_zeroes),
			usage: BufferUsage::COPY_DST | BufferUsage::INDEX,
		});

		PolylineBuffer { vertex_buffer, index_buffer, index_count: 0 }
	}

	/// Assumes bind group 0 is global uniforms
	pub fn render_buffers<'a>(&'a self, render_pass: &mut RenderPass<'a>, buffer: &'a PolylineBuffer) {
		render_pass.set_pipeline(&self.pipeline);
		render_pass.set_vertex_buffer(0, buffer.vertex_buffer.slice(..));
		render_pass.set_index_buffer(buffer.index_buffer.slice(..), IndexFormat::Uint32);
		render_pass.draw_indexed(0..buffer.index_count as u32, 0, 0..1)
	}
}

pub struct PolylineBuffer {
	vertex_buffer: Buffer,
	index_buffer: Buffer,
	index_count: u32,
}

impl PolylineBuffer {
	pub fn tessellate<'a>(&'a mut self, queue: &Queue, polylines: &[Polyline]) {
		let mut vertices: Vec<Vert> = Vec::new();
		let mut indices: Vec<u32> = Vec::new();

		for polyline in polylines {
			//Unfortunately I need a new path builder for each polyline
			//lyon doesn't support setting the thickness or color per-stroke, as far as I can tell??
			let mut path_builder = lyon::path::Path::builder();

			let mut point_iter = polyline.points.iter();
			let first = point_iter.next().expect("empty polyline?");
			path_builder.begin(lyon::geom::point(first.x, first.y));
			for next in point_iter {
				path_builder.line_to(lyon::geom::point(next.x, next.y));
			}
			path_builder.end(false); //no close

			let path = path_builder.build();

			use lyon::lyon_tessellation::*;

			let mut tess_out: VertexBuffers<Vert, u16> = VertexBuffers::new();
			let mut tess = StrokeTessellator::new();
			{
				tess.tessellate_path(
					&path,
					&StrokeOptions::default()
						.with_line_cap(LineCap::Butt)
						.with_line_join(LineJoin::Miter)
						.with_miter_limit(500.0)
						.with_line_width(polyline.thickness),
					&mut BuffersBuilder::new(&mut tess_out, |pos: StrokeVertex| Vert { position: pos.position().to_array(), color: polyline.color.into() }),
				)
				.expect("failed to tesselate");
			}

			//since i'll be shoving these into the same buffer, adjust the index buffer to point here
			//also map to u32 incase there's as hitton of lines (doubt it)
			let vert_count = vertices.len() as u32;
			let indices_u32 = tess_out.indices.into_iter().map(|x| (x as u32) + vert_count).collect::<Vec<_>>();

			vertices.extend_from_slice(&tess_out.vertices);
			indices.extend_from_slice(&indices_u32);
		}

		//great now fill the buffers on the GPU
		queue.write_buffer(&self.vertex_buffer, 0, &bytemuck::cast_slice(&vertices));
		queue.write_buffer(&self.index_buffer, 0, &bytemuck::cast_slice(&indices));
		self.index_count = indices.len() as u32;
	}
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Vert {
	position: [f32; 2],
	color: [f32; 3],
}

impl Vert {
	#[allow(dead_code)] //no, it's used, r-a
	pub const LAYOUT: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float2, 1 => Float3];
}
