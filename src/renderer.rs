use crate::math::{Vec3, Ray, Camera, Sphere};
use crate::shader::{sun_shader, rocky_shader, gas_giant_shader};
use minifb::{Window, WindowOptions};
use rand::Rng;
use nalgebra_glm as glm;
use std::f32::consts::PI;
use rayon::prelude::*;

pub const WIDTH: usize = 800;
pub const HEIGHT: usize = 600;

pub struct Scene {
    pub camera: Camera,
    pub sun: Sphere,
    pub rocky_planet: Sphere,
    pub gas_giant: Sphere,
    pub stars: Vec<(Vec3, f32, Vec3)>, // direction (unit), brightness, color
    pub sky_rotation: f32, // radians
    // orbital + spin parameters
    pub rocky_orbit_center: Vec3,
    pub rocky_orbit_radius: f32,
    pub rocky_orbit_speed: f32,
    pub rocky_spin_speed: f32,

    pub gas_orbit_center: Vec3,
    pub gas_orbit_radius: f32,
    pub gas_orbit_speed: f32,
    pub gas_spin_speed: f32,
}

impl Scene {
    pub fn new() -> Self {
        // Place the camera further back to improve composition and view
        let camera = Camera::new(
            glm::vec3(0.0, 0.0, 10.0), // moved back on Z
            glm::vec3(0.0, 0.0, 0.0),  // look at the sun at origin
            glm::vec3(0.0, 1.0, 0.0),
            45.0_f32.to_radians(),
            WIDTH as f32 / HEIGHT as f32,
        );

        // Put the sun at the origin so orbits rotate around it
        let sun = Sphere::new(glm::vec3(0.0, 0.0, 0.0), 1.0);
        // Create planets with placeholder centers; their actual intersection centers are
        // computed from orbital parameters in ray_intersect. Set initial visual positions
        // so debugging or unused code remains consistent.
        let rocky_planet = Sphere::new(glm::vec3(2.0, 0.0, 0.0), 0.5);
        let gas_giant = Sphere::new(glm::vec3(3.5, 0.0, 0.0), 0.8);

        // Generate random stars as directions on the unit sphere
        let mut rng = rand::thread_rng();
        let mut stars = Vec::new();
        for _ in 0..2000 {
            let z: f32 = rng.gen_range(-1.0..1.0);
            let theta: f32 = rng.gen_range(0.0..(2.0 * PI));
            let r = (1.0 - z * z).max(0.0).sqrt();
            let x = r * theta.cos();
            let y = r * theta.sin();
            let dir = glm::vec3(x, y, z);
            let brightness = rng.gen_range(0.5..1.0);
            // Slight color tint: many stars are slightly yellow/white/blue
            let t: f32 = rng.gen_range(0.0..1.0);
            let color = if t < 0.6 {
                glm::vec3(1.0, 0.95, 0.9) // warm-white
            } else if t < 0.9 {
                glm::vec3(0.9, 0.95, 1.0) // cool-white
            } else {
                glm::vec3(1.0, 0.9, 0.7) // more yellowish
            };
            stars.push((dir, brightness, color));
        }

        Scene {
            camera,
            sun,
            rocky_planet,
            gas_giant,
            stars,
            sky_rotation: 0.0,
            // rocky planet orbits around world origin at radius ~2.0
            rocky_orbit_center: glm::vec3(0.0, 0.0, 0.0),
            rocky_orbit_radius: 2.0,
            rocky_orbit_speed: 0.6, // radians per second
            rocky_spin_speed: 2.0, // spin radians per second
            // gas giant orbits a bit farther
            gas_orbit_center: glm::vec3(0.0, 0.0, 0.0),
            gas_orbit_radius: 3.5,
            gas_orbit_speed: 0.3,
            gas_spin_speed: 1.2,
        }
    }

    pub fn render(&self, time: f32) -> Vec<u32> {
        // Use ray_casting which parallelizes per-row for better performance
        self.ray_casting(time)
    }

    // Find closest intersection of ray with scene objects.
    // Returns Some((t, point, normal, type)) where type: 1=sun,2=rocky,3=gas
    // time in seconds drives orbital positions and spin
    fn ray_intersect(&self, ray: &Ray, time: f32) -> Option<(f32, Vec3, Vec3, u8)> {
        let mut closest_hit = f32::INFINITY;
        let mut hit_point = None;
        let mut hit_normal = None;
        let mut hit_type: u8 = 0;
        // Sun (static)
        if let Some(t) = self.sun.intersect(ray) {
            if t < closest_hit {
                closest_hit = t;
                let point = ray.origin + ray.direction * t;
                hit_point = Some(point);
                hit_normal = Some(self.sun.normal_at(&point));
                hit_type = 1;
            }
        }

        // Rocky planet: compute orbital center
        let rocky_angle = self.rocky_orbit_speed * time;
        let rocky_center = glm::vec3(
            self.rocky_orbit_center.x + self.rocky_orbit_radius * rocky_angle.cos(),
            self.rocky_orbit_center.y,
            self.rocky_orbit_center.z + self.rocky_orbit_radius * rocky_angle.sin(),
        );
        if let Some(t) = intersect_sphere(&rocky_center, self.rocky_planet.radius, ray) {
            if t < closest_hit {
                closest_hit = t;
                let point = ray.origin + ray.direction * t;
                hit_point = Some(point);
                hit_normal = Some(glm::normalize(&(point - rocky_center)));
                hit_type = 2;
            }
        }

        // Gas giant
        let gas_angle = self.gas_orbit_speed * time;
        let gas_center = glm::vec3(
            self.gas_orbit_center.x + self.gas_orbit_radius * gas_angle.cos(),
            self.gas_orbit_center.y,
            self.gas_orbit_center.z + self.gas_orbit_radius * gas_angle.sin(),
        );
        if let Some(t) = intersect_sphere(&gas_center, self.gas_giant.radius, ray) {
            if t < closest_hit {
                closest_hit = t;
                let point = ray.origin + ray.direction * t;
                hit_point = Some(point);
                hit_normal = Some(glm::normalize(&(point - gas_center)));
                hit_type = 3;
            }
        }

        if let (Some(p), Some(n)) = (hit_point, hit_normal) {
            Some((closest_hit, p, n, hit_type))
        } else {
            None
        }
    }

    // Parallel ray casting: render rows in parallel using rayon
    fn ray_casting(&self, time: f32) -> Vec<u32> {
        // For each row (y), produce a Vec<u32> for that row, then flatten
        let rows: Vec<Vec<u32>> = (0..HEIGHT).into_par_iter().map(|y| {
            let mut row = vec![0u32; WIDTH];
            for x in 0..WIDTH {
                let u = x as f32 / WIDTH as f32;
                let v = 1.0 - (y as f32 / HEIGHT as f32);
                let ray = self.camera.get_ray(u, v);

                let pixel = match self.ray_intersect(&ray, time) {
                    Some((_t, point, normal, hit_type)) => {
                        match hit_type {
                            1 => vec3_to_color(&sun_shader(&point, &normal, &ray.direction, time)),
                            2 => {
                                // rotate texture coordinates by planet spin
                                let rocky_spin = self.rocky_spin_speed * time;
                                let rocky_angle = -rocky_spin; // inverse to simulate texture rotation
                                let rocky_center = glm::vec3(
                                    self.rocky_orbit_center.x + self.rocky_orbit_radius * (self.rocky_orbit_speed * time).cos(),
                                    self.rocky_orbit_center.y,
                                    self.rocky_orbit_center.z + self.rocky_orbit_radius * (self.rocky_orbit_speed * time).sin(),
                                );
                                let rp = rotate_point_around_y(&point, &rocky_center, rocky_angle);
                                let rn = rotate_vector_around_y(&normal, rocky_angle);
                                vec3_to_color(&rocky_shader(&rp, &rn, &ray.direction, time))
                            }
                            3 => {
                                let gas_spin = self.gas_spin_speed * time;
                                let gas_angle = -gas_spin;
                                let gas_center = glm::vec3(
                                    self.gas_orbit_center.x + self.gas_orbit_radius * (self.gas_orbit_speed * time).cos(),
                                    self.gas_orbit_center.y,
                                    self.gas_orbit_center.z + self.gas_orbit_radius * (self.gas_orbit_speed * time).sin(),
                                );
                                let rp = rotate_point_around_y(&point, &gas_center, gas_angle);
                                let rn = rotate_vector_around_y(&normal, gas_angle);
                                vec3_to_color(&gas_giant_shader(&rp, &rn, &ray.direction, time))
                            }
                            _ => 0,
                        }
                    }
                    None => {
                        let color = self.skybox_color(&ray.direction, self.sky_rotation + time * 0.05);
                        vec3_to_color(&color)
                    }
                };

                row[x] = pixel;
            }
            row
        }).collect();

        // Flatten rows into a single buffer
        let mut buffer = Vec::with_capacity(WIDTH * HEIGHT);
        for r in rows {
            buffer.extend(r);
        }
        buffer
    }

    fn ray_color(&self, ray: &Ray, time: f32) -> u32 {
        // Background skybox sampling
        // If no object hit, sample skybox based on ray direction and current sky rotation

        // Check object intersections
        let mut closest_hit = f32::INFINITY;
        let mut hit_point = None;
        let mut hit_normal = None;
        let mut hit_type = 0; // 0=none, 1=sun, 2=rocky, 3=gas

        // Sun intersection
        if let Some(t) = self.sun.intersect(ray) {
            if t < closest_hit {
                closest_hit = t;
                let point = ray.origin + ray.direction * t;
                hit_point = Some(point);
                hit_normal = Some(self.sun.normal_at(&point));
                hit_type = 1;
            }
        }

        // Rocky planet intersection
        if let Some(t) = self.rocky_planet.intersect(ray) {
            if t < closest_hit {
                closest_hit = t;
                let point = ray.origin + ray.direction * t;
                hit_point = Some(point);
                hit_normal = Some(self.rocky_planet.normal_at(&point));
                hit_type = 2;
            }
        }

        // Gas giant intersection
        if let Some(t) = self.gas_giant.intersect(ray) {
            if t < closest_hit {
                let point = ray.origin + ray.direction * t;
                hit_point = Some(point);
                hit_normal = Some(self.gas_giant.normal_at(&point));
                hit_type = 3;
            }
        }

        match (hit_point, hit_normal, hit_type) {
            (Some(point), Some(normal), 1) => vec3_to_color(&sun_shader(&point, &normal, &ray.direction, time)),
            (Some(point), Some(normal), 2) => vec3_to_color(&rocky_shader(&point, &normal, &ray.direction, time)),
            (Some(point), Some(normal), 3) => vec3_to_color(&gas_giant_shader(&point, &normal, &ray.direction, time)),
            _ => {
                let color = self.skybox_color(&ray.direction, self.sky_rotation + time * 0.05);
                vec3_to_color(&color)
            }
        }
    }

    fn skybox_color(&self, dir: &Vec3, rotation: f32) -> Vec3 {
        // Rotate the view direction around Y by -rotation (so sky appears to rotate)
        let c = rotation.cos();
        let s = rotation.sin();
        let rx = c * dir.x + s * dir.z;
        let rz = -s * dir.x + c * dir.z;
        let rdir = glm::vec3(rx, dir.y, rz);

        // Sample stars: find if any star lies within a small angular radius of rdir
        let mut accum = glm::vec3(0.0, 0.0, 0.0);
        for (star_dir, brightness, color) in &self.stars {
            let d = glm::dot(&rdir, star_dir);
            // angular radius threshold (cos of angle). Smaller value => larger apparent star size
            let threshold = 0.9995_f32; // about ~0.03 rad
            if d > threshold {
                // soft falloff
                let intensity = ((d - threshold) / (1.0 - threshold)).powf(2.0) * *brightness;
                accum += color * intensity;
            }
        }

        // Add subtle gradient for space
        let space_color = glm::vec3(0.02, 0.03, 0.06);
        space_color + accum
    }

}

// Helper: sphere intersection with explicit center
fn intersect_sphere(center: &Vec3, radius: f32, ray: &Ray) -> Option<f32> {
    let oc = ray.origin - *center;
    let a = glm::dot(&ray.direction, &ray.direction);
    let b = 2.0 * glm::dot(&oc, &ray.direction);
    let c = glm::dot(&oc, &oc) - radius * radius;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        None
    } else {
        let t = (-b - disc.sqrt()) / (2.0 * a);
        if t > 0.0 { Some(t) } else { None }
    }
}

// Rotate a point around Y axis by angle (radians) around given center
fn rotate_point_around_y(p: &Vec3, center: &Vec3, angle: f32) -> Vec3 {
    let rel = p - *center;
    let c = angle.cos();
    let s = angle.sin();
    let x = rel.x * c - rel.z * s;
    let z = rel.x * s + rel.z * c;
    glm::vec3(x + center.x, rel.y + center.y, z + center.z)
}

// Rotate a direction vector around Y axis (no center)
fn rotate_vector_around_y(v: &Vec3, angle: f32) -> Vec3 {
    let c = angle.cos();
    let s = angle.sin();
    glm::vec3(v.x * c - v.z * s, v.y, v.x * s + v.z * c)
}


fn vec3_to_color(v: &Vec3) -> u32 {
    let r = (v.x.min(1.0).max(0.0) * 255.0) as u32;
    let g = (v.y.min(1.0).max(0.0) * 255.0) as u32;
    let b = (v.z.min(1.0).max(0.0) * 255.0) as u32;
    (r << 16) | (g << 8) | b
}