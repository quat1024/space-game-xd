use crate::render::GameRenderer;

pub struct Game {
	
}

impl Game {
	pub fn new() -> Self {
		Game {}
	}
	
	pub fn handle_input(&mut self, _event: &winit::event::WindowEvent) -> bool {
		false
	}
	
	pub fn update(&mut self) {
		//Do nothing
	}
}