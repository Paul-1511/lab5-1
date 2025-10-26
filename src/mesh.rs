pub struct Vertex {
pub pos: [f32; 3],
pub normal: [f32; 3],
}


pub struct Mesh {
pub vertices: Vec<Vertex>,
pub indices: Vec<[usize;3]>,
}


impl Mesh {
pub fn uv_sphere(u: usize, v: usize, radius: f32) -> Self {
let mut vertices = Vec::new();
for j in 0..=v {
let phi = std::f32::consts::PI * (j as f32) / (v as f32);
for i in 0..=u {
let theta = 2.0 * std::f32::consts::PI * (i as f32) / (u as f32);
let x = radius * phi.sin() * theta.cos();
let y = radius * phi.cos();
let z = radius * phi.sin() * theta.sin();
let len = (x*x + y*y + z*z).sqrt();
let nx = x / len; let ny = y / len; let nz = z / len;
vertices.push(Vertex { pos: [x,y,z], normal: [nx,ny,nz] });
}
}
let mut indices = Vec::new();
let width = u + 1;
for j in 0..v {
for i in 0..u {
let a = j * width + i;
let b = (j+1) * width + i;
indices.push([a, b, a+1]);
indices.push([b, b+1, a+1]);
}
}
Self { vertices, indices }
}
}