use anyhow::*;
use util::DeviceExt;
use wgpu::*;

use super::GameRendererBits;
use crate::asset_loader::AssetLoader;
use crate::world::LineSegment;
use crate::world::Polyline;

/// A thingie that helps you render lines by tesselatting them into triangles.
pub struct PolylineRenderer {
	buffers: Vec<PolylineBuffers>,
	pipeline: RenderPipeline,
}

struct PolylineBuffers {
	vertex_buffer: Buffer,
	index_buffer: Buffer,
	index_count: u16,
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

impl PolylineRenderer {
	#[allow(dead_code)] //no, it's used, r-a
	pub const MAX_POLYLINE_VERTS: u32 = 4096; //idk, how big can you make vertex buffers????

	pub fn new(game_renderer: &GameRendererBits, asset_loader: &AssetLoader) -> Result<Self> {
		let device = &game_renderer.device;

		let vert_module = asset_loader.create_shader_module(&device, "line.vert.spv").context("failed to load line renderer vert shader")?;
		let frag_module = asset_loader.create_shader_module(&device, "line.frag.spv").context("failed to load line renderer frag shader")?;

		let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
			label: Some("Line pipeline layout"),
			bind_group_layouts: &[],
			push_constant_ranges: &[],
		});

		//let buncha_zeroes = vec![0; Self::MAX_POLYLINE_VERTS as usize * std::mem::size_of::<Vert>()];

		// let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
		// 	label: Some("Line vertex buffer"),
		// 	contents: bytemuck::cast_slice(&buncha_zeroes),
		// 	usage: BufferUsage::COPY_DST | BufferUsage::VERTEX,
		// });

		// let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
		// 	label: Some("Line index buffer"),
		// 	contents: bytemuck::cast_slice(&buncha_zeroes),
		// 	usage: BufferUsage::COPY_DST | BufferUsage::INDEX,
		// });

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

		Ok(PolylineRenderer {
		    buffers: Vec::new(),
			pipeline,
		})
	}

	pub fn tesselate(&mut self, device: &Device, polylines: &[Polyline]) {
		self.buffers.clear(); //Drop all the buffers
		
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
			use lyon::lyon_tessellation::{self};

			let mut tess_out: VertexBuffers<Vert, u16> = VertexBuffers::new();
			let mut tess = StrokeTessellator::new();
			{
				tess.tessellate_path(
					&path,
					&StrokeOptions::default()
						.with_line_cap(LineCap::Round)
						.with_line_join(LineJoin::Round)
						.with_tolerance(0.001) //for now, because i'm working in NDC
						.with_line_width(polyline.thickness),
					&mut BuffersBuilder::new(&mut tess_out, |pos: StrokeVertex| Vert { position: pos.position().to_array(), color: polyline.color.into() }),
				)
				.expect("failed to tesselate");
			}
			
			let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: Some("Line vertex buffer"),
				contents: bytemuck::cast_slice(&tess_out.vertices),
				usage: BufferUsage::COPY_DST | BufferUsage::VERTEX,
			});
			
			let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
				label: Some("Line index buffer"),
				contents: bytemuck::cast_slice(&tess_out.indices),
				usage: BufferUsage::COPY_DST | BufferUsage::INDEX,
			});

			self.buffers.push(PolylineBuffers {
			    vertex_buffer,
			    index_buffer,
			    index_count: tess_out.indices.len() as u16,
			});
		}
	}
	
	pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
		render_pass.set_pipeline(&self.pipeline);
		for buffer_set in &self.buffers {
			render_pass.set_vertex_buffer(0, buffer_set.vertex_buffer.slice(..));
			render_pass.set_index_buffer(buffer_set.index_buffer.slice(..), IndexFormat::Uint16);
			render_pass.draw_indexed(0..buffer_set.index_count as u32, 0, 0..1)
		}
	}
}
