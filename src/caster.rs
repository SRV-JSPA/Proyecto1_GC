use crate::framebuffer::Framebuffer;
use crate::player::Player;
use nalgebra_glm::Vec2; // Asegúrate de importar Vec2

pub struct Intersect {
    pub distance: f32,
    pub impact: char
}

pub fn cast_ray(framebuffer: &mut Framebuffer, maze: &Vec<Vec<char>>, player: &Player, a: f32, block_size: usize, draw_line: bool) -> Intersect {
    let mut d = 0.0;
    let step_size = 5.0; 

    framebuffer.set_current_color(0xFFDDDD);

    loop {
        let cos = d * a.cos();
        let sin = d * a.sin();

        let x = (player.pos.x + cos) as usize;
        let y = (player.pos.y + sin) as usize;

        let i = x / block_size;
        let j = y / block_size;

        if j >= maze.len() || i >= maze[j].len() {
            return Intersect {
                distance: d,
                impact: ' '
            };
        }

        if maze[j][i] != ' ' {
            return Intersect {
                distance: d,
                impact: maze[j][i]
            };
        }

        if draw_line {
            framebuffer.point(x, y);
        }

        d += step_size;
    }
}

pub fn check_collision(maze: &Vec<Vec<char>>, player_pos: &Vec2, block_size: usize) -> bool {
    let player_x = player_pos.x as usize;
    let player_y = player_pos.y as usize;

    let i = player_x / block_size;
    let j = player_y / block_size;

    if j >= maze.len() || i >= maze[j].len() {
        return false;  // Fuera de límites
    }

    maze[j][i] != ' '
}
