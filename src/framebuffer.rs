use crate::color::Color;


pub struct Framebuffer {
pub width: usize,
pub height: usize,
pub pixels: Vec<Color>,
pub depth: Vec<f32>,
}


impl Framebuffer {
pub fn new(width: usize, height: usize) -> Self {
Self {
width,
height,
pixels: vec![Color::new(0.0,0.0,0.0); width * height],
depth: vec![f32::INFINITY; width * height],
}
}


pub fn set_pixel(&mut self, x: i32, y: i32, z: f32, color: Color) {
if x < 0 || y < 0 { return; }
let x = x as usize;
let y = y as usize;
if x >= self.width || y >= self.height { return; }
let idx = y * self.width + x;
if z < self.depth[idx] {
self.depth[idx] = z;
self.pixels[idx] = color;
}
}

	// Additive/emissive light: add color to pixel without depth testing.
	pub fn add_light(&mut self, x: i32, y: i32, color: Color) {
		if x < 0 || y < 0 { return; }
		let x = x as usize;
		let y = y as usize;
		if x >= self.width || y >= self.height { return; }
		let idx = y * self.width + x;
		self.pixels[idx] = self.pixels[idx].add(&color);
	}
}