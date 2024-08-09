mod framebuffer;
mod maze;
mod player;
mod caster;

use minifb::{Window, WindowOptions, Key};
use nalgebra_glm::Vec2;
use std::f32::consts::PI;
use std::time::{Duration, Instant};
use crate::framebuffer::Framebuffer;
use crate::maze::load_maze;
use crate::player::{Player, process_events};
use crate::caster::cast_ray;
use rodio::{Sink, OutputStream};
use std::fs::File;
use std::io::BufReader;
use gilrs::Gilrs;
use image::{DynamicImage, GenericImageView, Pixel};

const FUENTE: [[u8; 5]; 10] = [
    [0b01110, 0b10001, 0b10001, 0b10001, 0b01110], 
    [0b00100, 0b01100, 0b00100, 0b00100, 0b01110], 
    [0b01110, 0b10001, 0b00110, 0b01000, 0b11111], 
    [0b01110, 0b10001, 0b00110, 0b10001, 0b01110], 
    [0b00100, 0b01100, 0b10100, 0b11111, 0b00100], 
    [0b11111, 0b10000, 0b11110, 0b00001, 0b11110], 
    [0b01110, 0b10000, 0b11110, 0b10001, 0b01110], 
    [0b11111, 0b00010, 0b00100, 0b01000, 0b10000], 
    [0b01110, 0b10001, 0b01110, 0b10001, 0b01110], 
    [0b01110, 0b10001, 0b01111, 0b00001, 0b01110], 
];

struct Textures {
    wall1: DynamicImage,
    wall2: DynamicImage,
    wall3: DynamicImage,
}

impl Textures {
    fn load() -> Self {
        Self {
            wall1: image::open("images/cielo.jpg").unwrap(),
            wall2: image::open("images/ladrillo.jpeg").unwrap(),
            wall3: image::open("images/madera.jpg").unwrap(),
        }
    }
}

fn draw_digit(framebuffer: &mut Framebuffer, x: usize, y: usize, digit: u8) {
    if digit > 9 {
        return;
    }
    for (row, bits) in FUENTE[digit as usize].iter().enumerate() {
        for col in 0..5 {
            if bits & (1 << (4 - col)) != 0 {
                if x + col < framebuffer.width && y + row < framebuffer.height {
                    framebuffer.point(x + col, y + row);
                }
            }
        }
    }
}

fn draw_fps(framebuffer: &mut Framebuffer, fps: u32) {
    let mut fps_string = fps.to_string();
    let x_offset = 10;
    let y_offset = 10;
    let digit_width = 6;

    framebuffer.set_current_color(0xFFFFFF);

    for (i, ch) in fps_string.chars().enumerate() {
        if let Some(digit) = ch.to_digit(10) {
            draw_digit(framebuffer, x_offset + i * digit_width, y_offset, digit as u8);
        }
    }
}

fn draw_cell(framebuffer: &mut Framebuffer, xo: usize, yo: usize, block_size: usize, cell: char) {
    match cell {
        '+' => framebuffer.set_current_color(0xFF0000), // Rojo para '+'
        '|' => framebuffer.set_current_color(0x00FF00), // Verde para '|'
        '-' => framebuffer.set_current_color(0x0000FF), // Azul para '-'
        _ => return,
    }

    for x in xo..xo + block_size {
        for y in yo..yo + block_size {
            if x < framebuffer.width && y < framebuffer.height {
                framebuffer.point(x, y);
            }
        }
    }
}

fn render(framebuffer: &mut Framebuffer, player: &Player, x_offset: usize, y_offset: usize, scale: f32) {
    let maze = load_maze("./maze.txt");
    let block_size = (100.0 * scale) as usize;

    for row in 0..maze.len() {
        for col in 0..maze[row].len() {
            draw_cell(framebuffer, x_offset + col * block_size, y_offset + row * block_size, block_size, maze[row][col])
        }
    }

    framebuffer.set_current_color(0xFFDDD);
    let player_x = x_offset + (player.pos.x * scale) as usize;
    let player_y = y_offset + (player.pos.y * scale) as usize;
    if player_x < framebuffer.width && player_y < framebuffer.height {
        framebuffer.point(player_x, player_y);
    }

    let num_rays = 5;
    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);

        cast_ray(framebuffer, &maze, &player, a, block_size, false);
    }
}

fn render3d(framebuffer: &mut Framebuffer, player: &Player, textures: &Textures) {
    let maze = load_maze("./maze.txt");
    let block_size = 100;
    let num_rays = framebuffer.width;

    let hw = framebuffer.width as f32 / 2.0;
    let hh = framebuffer.height as f32 / 2.0;

    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        let intersect = cast_ray(framebuffer, &maze, &player, a, block_size, false);

        let distance_to_wall = intersect.distance;
        let distance_to_projection_plane = 80.0; 
        let stake_height = (hh / distance_to_wall) * distance_to_projection_plane;

        let stake_top = (hh - (stake_height / 2.0)) as usize;
        let stake_bottom = (hh + (stake_height / 2.0)) as usize;

        let texture = match intersect.impact {
            '+' => &textures.wall1,
            '|' => &textures.wall2,
            '-' => &textures.wall3,
            _ => &textures.wall1,
        };

        if stake_top < framebuffer.height && stake_bottom <= framebuffer.height {
            let texture_height = texture.height();
            for y in stake_top..stake_bottom {
                let texture_y = ((y - stake_top) as f32 / (stake_bottom - stake_top) as f32 * texture_height as f32) as u32;
                let pixel = texture.get_pixel((i % texture.width() as usize) as u32, texture_y);
                let rgba = pixel.to_rgba();
                let color = ((rgba[0] as u32) << 16) | ((rgba[1] as u32) << 8) | (rgba[2] as u32);
                framebuffer.set_current_color(color);
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
    let textures = Textures::load();


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

    let maze = load_maze("./maze.txt"); // Cargar el laberinto
    let block_size = 100; // El tamaño del bloque, ajusta según tu configuración

    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();

    let file = std::fs::File::open("assets/musica.wav").unwrap();
    sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());

    sink.play();

    let mut gilrs = Gilrs::new().unwrap();

    let mut last_time = Instant::now();
    let mut frame_count = 0;
    let mut fps = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let start_time = Instant::now();

        framebuffer.clear();

        process_events(&window, &mut player, &maze, block_size, &mut gilrs);

        render3d(&mut framebuffer, &player, &textures);

        let minimap_scale = 0.1;  
        let minimap_width = (framebuffer.width as f32 * minimap_scale) as usize;
        let minimap_height = (framebuffer.height as f32 * minimap_scale) as usize;
        let minimap_x_offset = framebuffer.width - minimap_width - 10;
        let minimap_y_offset = 10;

        render(&mut framebuffer, &player, minimap_x_offset, minimap_y_offset, minimap_scale);

        let duration = start_time.elapsed();
        let frame_time = duration.as_secs_f32();
        fps = (1.0 / frame_time) as u32;
        draw_fps(&mut framebuffer, fps);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);

        frame_count += 1;
        if frame_count % 60 == 0 {
            println!("FPS: {:.2}", fps);
        }

        if last_time.elapsed() >= Duration::from_secs(1) {
            last_time = Instant::now();
            frame_count = 0;
        }
    }

    sink.stop();
}
