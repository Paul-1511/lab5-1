#[derive(Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}


impl Color {
pub fn new(r: f32, g: f32, b: f32) -> Self {
Self { r, g, b }
}


pub fn clamp_u8(&self) -> [u8; 3] {
fn c(v: f32) -> u8 {
let v = if v.is_finite() { v } else { 0.0 };
(v.max(0.0).min(1.0) * 255.0 + 0.5) as u8
}
[c(self.r), c(self.g), c(self.b)]
}


pub fn mul_scalar(&self, s: f32) -> Self {
Self::new(self.r * s, self.g * s, self.b * s)
}


pub fn add(&self, o: &Color) -> Self {
Self::new(self.r + o.r, self.g + o.g, self.b + o.b)
}
}

// handy constants
// (Removed unused BLACK/WHITE constants to avoid dead-code warnings.)