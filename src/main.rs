mod framebuffer;
mod maze;
mod player;
mod caster;

use minifb::{Window, WindowOptions, Key};
use nalgebra_glm::Vec2;
use std::f32::consts::PI;
use std::time::Duration;
use crate::framebuffer::Framebuffer;
use crate::maze::load_maze;
use crate::player::{Player, process_events};
use crate::caster::cast_ray;
use std::io::BufReader;
use rodio::OutputStream;

fn draw_cell(framebuffer: &mut Framebuffer, xo: usize, yo: usize, block_size: usize, cell: char) {
    if cell == ' ' {
        return;
    }

    framebuffer.set_current_color(0xFFDDDD);

    for x in xo..xo + block_size {
        for y in yo..yo + block_size {
            if x < framebuffer.width && y < framebuffer.height {
                framebuffer.point(x, y);
            }
        }
    }
}

fn render(framebuffer: &mut Framebuffer, player: &Player) {
    let maze = load_maze("./maze.txt");
    let block_size = 100;

    for row in 0..maze.len() {
        for col in 0..maze[row].len() {
            draw_cell(framebuffer, col * block_size, row * block_size, block_size, maze[row][col])
        }
    }

    framebuffer.set_current_color(0xFFDDD);
    if player.pos.x >= 0.0 && player.pos.x < framebuffer.width as f32 && player.pos.y >= 0.0 && player.pos.y < framebuffer.height as f32 {
        framebuffer.point(player.pos.x as usize, player.pos.y as usize);
    }

    let num_rays = 5;
    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);

        cast_ray(framebuffer, &maze, &player, a, block_size, false);
    }
}

fn render3d(framebuffer: &mut Framebuffer, player: &Player) {
    let maze = load_maze("./maze.txt");
    let block_size = 100;
    let num_rays = framebuffer.width;

    let hw = framebuffer.width as f32 / 2.0;
    let hh = framebuffer.height as f32 / 2.0;

    framebuffer.set_current_color(0xFFFFFF);

    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        let intersect = cast_ray(framebuffer, &maze, &player, a, block_size, false);

        let distance_to_wall = intersect.distance;
        let distance_to_projection_plane = 80.0; 
        let stake_height = (hh / distance_to_wall) * distance_to_projection_plane;

        let stake_top = (hh - (stake_height / 2.0)) as usize;
        let stake_bottom = (hh + (stake_height / 2.0)) as usize;

        framebuffer.set_current_color(0xFFFFFF);

        if stake_top < framebuffer.height && stake_bottom <= framebuffer.height {
            for y in stake_top..stake_bottom {
                framebuffer.point(i, y);
            }
        }
    }
}

fn main() {
    let window_width = 1300;
    let window_height = 900;
    let framebuffer_width = 1300;
    let framebuffer_height = 900;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);

    let mut window = Window::new(
        "Maze Runner",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    framebuffer.set_background_color(0x333355);

    let mut player = Player {
        pos: Vec2::new(150.0, 150.0),
        a: PI / 3.0,
        fov: PI / 3.0
    };

    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();

    let file = std::fs::File::open("assets/musica.wav").unwrap();
    sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());

    sink.play();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        framebuffer.clear();

        process_events(&window, &mut player);

        render3d(&mut framebuffer, &player);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }

    // Detener la música cuando se cierra la ventana
    sink.stop();
}
