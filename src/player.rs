use nalgebra_glm::Vec2;
use std::f32::consts::PI;
use minifb::{Window, Key};

pub struct Player {
    pub pos: Vec2,
    pub a: f32, 
    pub fov: f32, 
}

pub fn process_events(window: &Window, player: &mut Player) {
    const MOVE_SPEED: f32 = 5.0;
    const ROTATION_SPEED: f32 = PI / 35.0;

    if window.is_key_down(Key::Left) {
        player.a -= ROTATION_SPEED;
    }
    if window.is_key_down(Key::Right) {
        player.a += ROTATION_SPEED;
    }
    if window.is_key_down(Key::Up) {
        let direction = Vec2::new(player.a.cos(), player.a.sin());
        player.pos += direction * MOVE_SPEED;
    }
    if window.is_key_down(Key::Down) {
        let direction = Vec2::new(player.a.cos(), player.a.sin());
        player.pos -= direction * MOVE_SPEED;
    }
}
