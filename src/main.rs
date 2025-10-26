mod math;
mod renderer;
mod shader;

use minifb::{Key, Window, WindowOptions};
use renderer::{Scene, WIDTH, HEIGHT};

fn main() {
    let mut window = Window::new(
        "Sistema Solar 3D",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: true,
            scale: minifb::Scale::X1,
            ..WindowOptions::default()
        },
    )
    .unwrap();


    let mut scene = Scene::new();
    let mut time = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Rotate sky with left/right arrow keys
        if window.is_key_down(Key::Left) {
            scene.sky_rotation -= 0.03;
        }
        if window.is_key_down(Key::Right) {
            scene.sky_rotation += 0.03;
        }

        let buffer = scene.render(time);
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        time += 0.016; // Aproximadamente 60 FPS
    }
}