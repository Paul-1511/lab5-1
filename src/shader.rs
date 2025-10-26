use nalgebra_glm as glm;
use crate::math::Vec3;

// Light position in world space
const LIGHT_POS: Vec3 = Vec3::new(-2.0, 0.0, -2.0);

fn saturate(x: f32) -> f32 {
	x.max(0.0).min(1.0)
}

fn tri_noise(p: &Vec3, freq: f32, t: f32) -> f32 {
	let s = (p.x * freq + t).sin();
	let c = (p.y * freq * 0.7 - t * 0.5).cos();
	let d = (p.z * freq * 0.3 + t * 0.3).sin();
	saturate((s * c * d + 1.0) * 0.5)
}

pub fn sun_shader(world_pos: &Vec3, normal: &Vec3, view: &Vec3, time: f32) -> Vec3 {
	// Enhanced plasma-like colors
	let white_hot = glm::vec3(1.0, 1.0, 0.98);
	let yellow_bright = glm::vec3(1.0, 0.95, 0.2);
	let yellow_deep = glm::vec3(0.95, 0.75, 0.15);
	let orange_dark = glm::vec3(0.9, 0.4, 0.0);
    
	// Multi-layered noise for complex texture
	let spots = tri_noise(world_pos, 35.0, time * 2.5);
	let yellow_var = tri_noise(world_pos, 18.0, time * 1.4);
	let streaks = tri_noise(world_pos, 15.0, time * 1.2);
    
	// Create more pronounced irregular streaks
	let streak_pattern = (streaks * 1.5 + 
						 (world_pos.x * 10.0 + time).sin() * 0.5 + 
						 (world_pos.y * 8.0 - time * 0.7).cos() * 0.4 +
						 (world_pos.z * 6.0 + time * 0.3).sin() * 0.3).abs();
    
	let white_intensity = saturate((spots - 0.8) * 4.0);
	let orange_intensity = saturate((streak_pattern - 0.4) * 2.5);
    
	// Dynamic yellow base with variation
	let yellow_mix = saturate(yellow_var * 1.2);
	let base_yellow = yellow_bright * (1.0 - yellow_mix) + yellow_deep * yellow_mix;
	let mut combined = base_yellow * (0.7 + 0.5 * tri_noise(world_pos, 12.0, time));
    
	// Add white spots
	if white_intensity > 0.1 {
		combined += white_hot * (white_intensity * 0.9);
	}
    
	// Add darker orange streaks
	if orange_intensity > 0.0 {
		combined += orange_dark * (orange_intensity * 0.9);
	}

	// Add rim effect
	let rim = (1.0 - saturate(glm::dot(view, normal))).powf(3.0) * 0.4;
	combined += orange_dark * rim;

	// Final variation
	combined * (0.8 + 0.4 * tri_noise(world_pos, 25.0, time * 1.8))
}

pub fn rocky_shader(world_pos: &Vec3, normal: &Vec3, view: &Vec3, _time: f32) -> Vec3 {
	let light_dir = glm::normalize(&(LIGHT_POS - world_pos));
	let n_dot_l = saturate(glm::dot(normal, &light_dir));
    
	// Base colors
	let rock_dark = glm::vec3(0.35, 0.23, 0.12);
	let rock_light = glm::vec3(0.5, 0.5, 0.48);
    
	// Create banded pattern based on spherical coordinates
	let lat = normal.y;
	let bands = 0.5 + 0.5 * (lat * 20.0).sin();
    
	// Mix colors based on bands
	let base_color = rock_dark * (1.0 - bands) + rock_light * bands;
    
	// Add craters using noise
	let crater = tri_noise(world_pos, 30.0, 0.0);
	let crater_mask = saturate((crater - 0.5) * 3.0);
    
	// Lighting
	let ambient = 0.3;
	let diffuse = n_dot_l * 0.7;
	let mut final_color = base_color * (ambient + diffuse);
    
	// Darken craters
	final_color *= 1.0 - crater_mask * 0.3;
    
	// Add specular highlight
	let half_vec = glm::normalize(&(light_dir + view));
	let spec = saturate(glm::dot(normal, &half_vec)).powf(32.0) * 0.4;
	final_color += glm::vec3(spec, spec, spec);

	// Add rim lighting
	let rim = (1.0 - saturate(glm::dot(normal, view))).powf(3.0) * 0.14;
	final_color += glm::vec3(0.9, 0.75, 0.55) * rim;

	final_color
}

pub fn gas_giant_shader(world_pos: &Vec3, normal: &Vec3, view: &Vec3, time: f32) -> Vec3 {
	// Calculate bands based on latitude (y coordinate in normal space)
	let lat = normal.y;
	let lon = normal.x.atan2(normal.z);
    
	// Create flowing band pattern
	let flow = lon * 6.0 + time * 0.8 + (lat * 10.0).sin() * 0.5;
	let turbulence = tri_noise(world_pos, 8.0, time * 0.5);
	let bands = (flow + turbulence * 2.0).sin() * 0.5 + 0.5;
    
	// Colors for the bands
	let color1 = glm::vec3(0.95, 0.78, 0.48); // Light band
	let color2 = glm::vec3(0.25, 0.55, 0.85); // Dark band
    
	// Mix colors based on bands
	let base_color = color1 * bands + color2 * (1.0 - bands);
    
	// Lighting
	let light_dir = glm::normalize(&(LIGHT_POS - world_pos));
	let n_dot_l = saturate(glm::dot(normal, &light_dir));
	let ambient = 0.3;
	let diffuse = n_dot_l * 0.7;
    
	// Add specular highlight
	let half_vec = glm::normalize(&(light_dir + view));
	let spec = saturate(glm::dot(normal, &half_vec)).powf(16.0) * 0.3;
    
	// Combine lighting
	let mut final_color = base_color * (ambient + diffuse);
	final_color += glm::vec3(spec, spec, spec);
    
	// Add atmospheric rim effect
	let rim = (1.0 - saturate(glm::dot(normal, view))).powf(3.0) * 0.2;
	final_color += glm::vec3(0.6, 0.7, 0.95) * rim;
    
	final_color
}

