use std::cmp::min;
use std::f32::consts::PI;

use ctru::{
    prelude::*,
    services::gfx::{Flush, RawFrameBuffer, Screen, Swap},
};

struct RayCasting {
    increment_angle: f32,
    precision: u8,
}

struct Player {
    fov: u8,
    half_fov: u8,
    x: f32,
    y: f32,
    angle: i16,
}

const SCREEN_WIDTH: u16 = 400;
const SCREEN_HEIGHT: u16 = 240;
const SCALE: u32 = 6;

const SQUARE_COLOR_R: u8 = 255;
const SQUARE_COLOR_G: u8 = 0;
const SQUARE_COLOR_B: u8 = 0;

pub static SQUARE_COLOR: [u8; 3] = [SQUARE_COLOR_B, SQUARE_COLOR_G, SQUARE_COLOR_R];
pub static SQUARE_COLOR2: [u8; 3] = [100, 0, 0];
pub static SQUARE_COLOR3: [u8; 3] = [0, 100, 0];

pub struct TextureBitmap {
    size: u8,
    bit_map: [[u8; 8]; 8],
    colors: [[u8; 3]; 2],
}

pub static GREY_BRICK: TextureBitmap = TextureBitmap {
    size: 8,
    bit_map: [
        [1, 1, 1, 1, 1, 1, 1, 1],
        [0, 0, 0, 1, 0, 0, 0, 1],
        [1, 1, 1, 1, 1, 1, 1, 1],
        [0, 1, 0, 0, 0, 1, 0, 0],
        [1, 1, 1, 1, 1, 1, 1, 1],
        [0, 0, 0, 1, 0, 0, 0, 1],
        [1, 1, 1, 1, 1, 1, 1, 1],
        [0, 1, 0, 0, 0, 1, 0, 0],
    ],
    colors: [[232, 241, 255], [199, 195, 194]],
};

pub static PLACEHOLDER: TextureBitmap = TextureBitmap {
    size: 8,
    bit_map: [
        [1, 1, 1, 1, 0, 0, 0, 0],
        [1, 1, 1, 1, 0, 0, 0, 0],
        [1, 1, 1, 1, 0, 0, 0, 0],
        [1, 1, 1, 1, 0, 0, 0, 0],
        [0, 0, 0, 0, 1, 1, 1, 1],
        [0, 0, 0, 0, 1, 1, 1, 1],
        [0, 0, 0, 0, 1, 1, 1, 1],
        [0, 0, 0, 0, 1, 1, 1, 1],
    ],
    colors: [[128, 0, 128], [0, 0, 0]],
};

fn main() {
    if SCALE == 0 {
        panic!();
    }
    let apt = Apt::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let mut gfx = Gfx::new().unwrap();
    let mut player = Player {
        fov: 60,
        half_fov: 30,
        x: 2.0,
        y: 2.0,
        angle: 45,
    };
    let mut ray_cast = RayCasting {
        increment_angle: player.fov as f32 / SCREEN_WIDTH as f32,
        precision: 64,
    };
    let map: [[u8; 10]; 10] = [
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        [1, 0, 0, 1, 1, 0, 1, 0, 0, 1],
        [1, 0, 0, 1, 0, 0, 1, 0, 0, 1],
        [1, 0, 0, 1, 0, 0, 1, 0, 0, 1],
        [1, 0, 0, 1, 0, 1, 1, 0, 0, 1],
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        [1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    ];
    let mut keys: KeyPad;
    let mut angle: f32;
    let mut pcos: f32;
    let mut psin: f32;
    let mut new_x: f32;
    let mut new_y: f32;
    let mut check_x: usize;
    let mut check_y: usize;
    while apt.main_loop() {
        hid.scan_input();
        if hid.keys_down().contains(KeyPad::START) {
            break;
        }
        keys = hid.keys_held();
        if keys.contains(KeyPad::DPAD_LEFT) {
            player.angle -= 2;
            if player.angle <= 0 {
                player.angle = 359;
            }
            {
                let top_screen = gfx.top_screen.get_mut();
                let frame_buffer = top_screen.raw_framebuffer();
                (player, ray_cast) = ray_casting(player, ray_cast, map, &frame_buffer);
                top_screen.flush_buffers();
                top_screen.swap_buffers();
            }
        } else if keys.contains(KeyPad::DPAD_RIGHT) {
            player.angle += 2;
            if player.angle >= 360 {
                player.angle = 1;
            }
            {
                let top_screen = gfx.top_screen.get_mut();
                let frame_buffer = top_screen.raw_framebuffer();
                (player, ray_cast) = ray_casting(player, ray_cast, map, &frame_buffer);
                top_screen.flush_buffers();
                top_screen.swap_buffers();
            }
        } else if keys.contains(KeyPad::DPAD_UP) {
            angle = player.angle as f32;
            pcos = 0.04 * deg_to_rad(&angle).cos();
            psin = 0.04 * deg_to_rad(&angle).sin();
            new_x = player.x + pcos;
            new_y = player.y + psin;
            check_x = (new_x + pcos * 20.0) as usize;
            check_y = (new_y + psin * 20.0) as usize;
            if map[check_y][player.x as usize] == 0 {
                player.y = new_y;
            }
            if map[player.y as usize][check_x] == 0 {
                player.x = new_x;
            }
            {
                let top_screen = gfx.top_screen.get_mut();
                let frame_buffer = top_screen.raw_framebuffer();
                (player, ray_cast) = ray_casting(player, ray_cast, map, &frame_buffer);
                top_screen.flush_buffers();
                top_screen.swap_buffers();
            }
        } else if keys.contains(KeyPad::DPAD_DOWN) {
            angle = player.angle as f32;
            pcos = 0.04 * deg_to_rad(&angle).cos();
            psin = 0.04 * deg_to_rad(&angle).sin();
            if map[(psin + player.y) as usize][(pcos + player.x) as usize] == 0 {
                player.x -= pcos;
                player.y -= psin;
            }
            {
                let top_screen = gfx.top_screen.get_mut();
                let frame_buffer = top_screen.raw_framebuffer();
                (player, ray_cast) = ray_casting(player, ray_cast, map, &frame_buffer);
                top_screen.flush_buffers();
                top_screen.swap_buffers();
            }
        }
        gfx.wait_for_vblank();
    }
}

fn deg_to_rad(deg: &f32) -> f32 {
    return *deg * PI / 180.0;
}

fn draw_filled_rec(
    frame_buffer: &RawFrameBuffer<'_>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: &[u8; 3],
    texture: bool,
    texture_slice: &[u8],
    frame_buffer_slice: &mut [u8],
) {

    let mut chosen_color = color;
    for i in 0..min(height, 240) {
        for a in 0..width {
            let new_x = x + a;
            let new_y = y + i;
            if texture {
                chosen_color = &PLACEHOLDER.colors[texture_slice[i as usize] as usize];
            }
            //if new_x < frame_buffer.height as u32 && new_y < frame_buffer.width as u32 {
                let pixel_index = ((new_x) * frame_buffer.width as u32 + (new_y)) as usize * 3;
                frame_buffer_slice[pixel_index..pixel_index + 3].copy_from_slice(chosen_color);
            //}
        }
    }
}

struct Ray {
    x: f32,
    y: f32,
}

fn ray_casting(
    player: Player,
    ray: RayCasting,
    map: [[u8; 10]; 10],
    frame_buffer: &RawFrameBuffer<'_>,
) -> (Player, RayCasting) {
    let mut ray_angle = player.angle as f32 - player.half_fov as f32;
    let mut line_vec: Vec<u8> = Vec::with_capacity(240);
    let frame_buffer_slice = unsafe {
        std::slice::from_raw_parts_mut(
            frame_buffer.ptr,
            ((frame_buffer.height * frame_buffer.width) * 3) as usize,
        )
    };
    for i in 1..SCREEN_WIDTH / SCALE as u16 {
        let mut ray_struct = Ray {
            x: player.x,
            y: player.y,
        };
        //increment
        ray_angle += ray.increment_angle * SCALE as f32;
        let ray_rad = deg_to_rad(&ray_angle);
        let pres = ray.precision as f32;
        let ray_cos = ray_rad.cos() / pres;
        let ray_sin = ray_rad.sin() / pres;
        // wall check
        let mut wall = 0;
        while wall == 0 {
            ray_struct.x += ray_cos;
            ray_struct.y += ray_sin;
            wall = map[ray_struct.y as usize][ray_struct.x as usize];
        }
        let mut distance =
            ((player.x - ray_struct.x).powi(2) + (player.y - ray_struct.y).powi(2)).sqrt();
        let panglef = ray_angle - player.angle as f32;
        // wall height
        distance = distance * (deg_to_rad(&panglef)).cos();
        let wall_height = (120.0 / distance);
        let mut wall_start = 120 - wall_height as u32;
        if distance <= 1.0 {
            wall_start = 1;
        }
        let texture_pos_x =
            ((GREY_BRICK.size as f32 * (ray_struct.x + ray_struct.y)) % GREY_BRICK.size as f32) as usize;
        //let texture_pos_x = 0;
        let texture_slice = get_texture_slice(texture_pos_x);
        let true_height = wall_height as u32 * 2;
        line_vec = get_vert_tex_map(&texture_slice, &true_height, line_vec);
        let v_tex_slice: &[u8];
        let n = line_vec.len();
        if n <= 240 {
            v_tex_slice = &line_vec[..];
        } else {
            let overflow = n - 240;
            let half_overflow: usize;
            if overflow % 2 == 0 {
                half_overflow = overflow / 2;
            } else {
                half_overflow = (overflow + 1) / 2;
            }
            v_tex_slice = &line_vec[half_overflow..(n - half_overflow)];
        }
        draw_filled_rec(
            frame_buffer,
            i as u32 * SCALE,
            wall_start - 1,
            SCALE,
            wall_height as u32 * 2,
            &SQUARE_COLOR,
            true,
            v_tex_slice,
            frame_buffer_slice,
        );
        if wall_height < 120.0 {
            let empty_slice: &[u8] = &[];
            let empty_slice2: &[u8] = &[];
            draw_filled_rec(
                frame_buffer,
                i as u32 * SCALE,
                0,
                SCALE,
                wall_start,
                &SQUARE_COLOR3,
                false,
                empty_slice,
                frame_buffer_slice,
            );
            draw_filled_rec(
                frame_buffer,
                i as u32 * SCALE,
                wall_start + (wall_height as u32 * 2),
                SCALE,
                SCREEN_HEIGHT as u32 - (wall_start + (wall_height as u32 * 2)),
                &SQUARE_COLOR2,
                false,
                empty_slice2,
                frame_buffer_slice,
            );
        }
        line_vec.clear();
        line_vec.reserve(240);
    }
    return (player, ray);
}

fn get_texture_slice(texture_pos_x: usize) -> Vec<u8> {
    let mut tslice: Vec<u8> = Vec::new();
    for i in PLACEHOLDER.bit_map.iter() {
        tslice.push(i[texture_pos_x]);
    }
    tslice
}

fn get_vert_tex_map<'a>(t_slice: &'a Vec<u8>, height: &'a u32, mut v_map: Vec<u8>) -> Vec<u8> {
    let mut prev_index = 0;
    let step = *height as f32 / 8.0;
    for i in 1..height + 1 {
        let x = (i as f32 / step) as usize;
        if x > prev_index {
            prev_index = x;
            if prev_index > 7 {
                prev_index = 7;
            }
        }
        v_map.push(t_slice[prev_index]);
    }
    return v_map;
}
