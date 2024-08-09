use nalgebra_glm::Vec2;
use std::f32::consts::PI;
use minifb::{Window, Key, MouseButton};
use gilrs::{Gilrs, Button, Event, EventType, Axis};
use crate::caster::check_collision; // Asegúrate de importar check_collision
use crate::framebuffer::Framebuffer;

pub struct Player {
    pub pos: Vec2,
    pub a: f32,
    pub fov: f32,
    pub mouse_sensitivity: f32, // Añadido el campo de sensibilidad del ratón
}

pub fn process_events(
    window: &Window,
    player: &mut Player,
    maze: &Vec<Vec<char>>,
    block_size: usize,
    gilrs: &mut Gilrs,
    mouse_position: &mut Vec2, // Añadido para almacenar la posición del mouse
) {
    const MOVE_SPEED: f32 = 5.0;
    const ROTATION_SPEED: f32 = PI / 35.0;
    const JOYSTICK_SENSITIVITY: f32 = 0.1;

    // Obtener la posición actual del ratón
    let current_mouse_position = window.get_mouse_pos(minifb::MouseMode::Pass).unwrap_or((0.0, 0.0));
    let current_mouse_pos = Vec2::new(current_mouse_position.0 as f32, current_mouse_position.1 as f32);

    // Calcular el delta del ratón
    let mouse_delta = current_mouse_pos - *mouse_position;
    *mouse_position = current_mouse_pos;

    // Manejar eventos de teclado
    if window.is_key_down(Key::Left) {
        player.a -= ROTATION_SPEED;
    }
    if window.is_key_down(Key::Right) {
        player.a += ROTATION_SPEED;
    }
    if window.is_key_down(Key::Up) {
        let direction = Vec2::new(player.a.cos(), player.a.sin());
        let new_pos = player.pos + direction * MOVE_SPEED;
        if !check_collision(maze, &new_pos, block_size) {
            player.pos = new_pos;
        }
    }
    if window.is_key_down(Key::Down) {
        let direction = Vec2::new(player.a.cos(), player.a.sin());
        let new_pos = player.pos - direction * MOVE_SPEED;
        if !check_collision(maze, &new_pos, block_size) {
            player.pos = new_pos;
        }
    }

    // Variables para almacenar el movimiento de las palancas
    let mut joystick_move_x = 0.0;
    let mut joystick_move_y = 0.0;
    let mut joystick_rotate = 0.0;

    // Manejar eventos del gamepad
    while let Some(Event { id: _, event, time: _ }) = gilrs.next_event() {
        match event {
            EventType::ButtonPressed(Button::DPadLeft, _) => {
                player.a -= ROTATION_SPEED;
            }
            EventType::ButtonPressed(Button::DPadRight, _) => {
                player.a += ROTATION_SPEED;
            }
            EventType::ButtonPressed(Button::DPadUp, _) => {
                let direction = Vec2::new(player.a.cos(), player.a.sin());
                let new_pos = player.pos + direction * MOVE_SPEED;
                if !check_collision(maze, &new_pos, block_size) {
                    player.pos = new_pos;
                }
            }
            EventType::ButtonPressed(Button::DPadDown, _) => {
                let direction = Vec2::new(player.a.cos(), player.a.sin());
                let new_pos = player.pos - direction * MOVE_SPEED;
                if !check_collision(maze, &new_pos, block_size) {
                    player.pos = new_pos;
                }
            }
            EventType::AxisChanged(Axis::LeftStickX, value, _) => {
                joystick_move_x = value;
            }
            EventType::AxisChanged(Axis::LeftStickY, value, _) => {
                joystick_move_y = value;
            }
            EventType::AxisChanged(Axis::RightStickX, value, _) => {
                joystick_rotate = value;
            }
            _ => {}
        }
    }

    // Aplicar movimiento de las palancas
    if joystick_rotate.abs() > JOYSTICK_SENSITIVITY {
        player.a += joystick_rotate * ROTATION_SPEED;
    }
    if joystick_move_x.abs() > JOYSTICK_SENSITIVITY || joystick_move_y.abs() > JOYSTICK_SENSITIVITY {
        let direction = Vec2::new(player.a.cos(), player.a.sin());
        let strafe_direction = Vec2::new(player.a.sin(), -player.a.cos());
        let new_pos = player.pos
            + direction * joystick_move_y * MOVE_SPEED
            + strafe_direction * joystick_move_x * MOVE_SPEED;
        if !check_collision(maze, &new_pos, block_size) {
            player.pos = new_pos;
        }
    }

    // Aplicar el movimiento del ratón
    if mouse_delta.x.abs() > 1.0 || mouse_delta.y.abs() > 1.0 {
        player.a -= mouse_delta.x * player.mouse_sensitivity;
    }
}
