pub mod chess_board;

use macroquad::{
    camera::{self, Camera2D},
    color::colors,
    input::{self, KeyCode},
    shapes, window,
};

#[macroquad::main("Infinite Armada Chess")]
async fn main() {
    const SCREEN_HEIGHT: f32 = 8.0;

    let mut world_camera = Camera2D {
        zoom: [1.0, -2.0 / SCREEN_HEIGHT].into(),
        offset: [-1.0, -1.0].into(),
        ..Default::default()
    };

    let mut fullscreen = false;

    loop {
        if input::is_key_pressed(KeyCode::F11) {
            fullscreen ^= true;
            window::set_fullscreen(fullscreen);
        }

        update_camera_aspect_ratio(&mut world_camera);
        camera::set_camera(&world_camera);

        shapes::draw_rectangle(0.0, 0.0, 1.0, 1.0, colors::WHITE);

        window::next_frame().await;
    }
}

fn update_camera_aspect_ratio(camera: &mut Camera2D) {
    if let Some((_, _, size_x, size_y)) = camera.viewport {
        camera.zoom.x = camera.zoom.y.abs() * size_y as f32 / size_x as f32;
    } else {
        camera.zoom.x = camera.zoom.y.abs() * window::screen_height() / window::screen_width();
    }
}
