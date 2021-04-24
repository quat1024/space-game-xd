use anyhow::*;
use winit::dpi::LogicalSize;
use winit::event::*;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::Window;
use winit::window::WindowBuilder;

use crate::game::Game;
use crate::render::GameRenderer;

pub struct GameWindow {
	pub event_loop: EventLoop<()>,
	pub window: Window,
}

impl GameWindow {
	pub fn new(title: &'static str, logical_width: u32, logical_height: u32) -> Result<Self> {
		let event_loop = EventLoop::new();

		let window = WindowBuilder::new()
			.with_resizable(true)
			.with_title(title)
			.with_inner_size(winit::dpi::LogicalSize::new(logical_width, logical_height))
			.build(&event_loop)
			.expect("couldn't create window");

		Ok(GameWindow { event_loop, window })
	}

	pub fn run_loop(self, mut game: Game, mut renderer: GameRenderer) {
		let window = self.window;

		self.event_loop.run(move |event, _window_target, control_flow| match event {
			Event::WindowEvent { window_id, event } if window_id == window.id() => {
				if !game.handle_input(&event) {
					match event {
						WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
						WindowEvent::KeyboardInput {
							input: KeyboardInput { state: ElementState::Pressed, virtual_keycode: Some(VirtualKeyCode::Escape), .. },
							..
						} => *control_flow = ControlFlow::Exit,
						WindowEvent::Resized(physical_size) => {
							renderer.resize(physical_size);
						},
						WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
							renderer.resize(*new_inner_size);
						},
						_ => (),
					}
				}
			},
			Event::MainEventsCleared => {
				game.update();

				match renderer.render(&mut game) {
					Ok(_) => (),
					Err(wgpu::SwapChainError::Lost | wgpu::SwapChainError::Outdated) => renderer.recreate_swap_chain(),
					Err(wgpu::SwapChainError::OutOfMemory) => {
						eprintln!("Swap chain error: {:?}", wgpu::SwapChainError::OutOfMemory);
						*control_flow = ControlFlow::Exit
					},
					Err(something_else) => eprintln!("Swap chain error: {:?}", something_else),
				}
			},
			_ => (),
		});
	}
}
