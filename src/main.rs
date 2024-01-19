use std::borrow::BorrowMut;
use std::cmp::min;
use std::f32::consts::PI;
use std::fs::File;
use std::io::{self, Write};
use std::time::SystemTime;
use std::time::{Duration, Instant};

use ctru::{
    prelude::*,
    services::gfx::{self, Flush, RawFrameBuffer, Screen, Swap, TopScreen, TopScreenLeft},
};
use nalgebra::{matrix, Matrix, Point2};

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
const SCALE: u32 = 4;

const SQUARE_COLOR_R: u8 = 255;
const SQUARE_COLOR_G: u8 = 0;
const SQUARE_COLOR_B: u8 = 0;

pub static SQUARE_COLOR: [u8; 3] = [SQUARE_COLOR_B, SQUARE_COLOR_G, SQUARE_COLOR_R];
pub static SQUARE_COLOR2: [u8; 3] = [100, 0, 0];
pub static SQUARE_COLOR3: [u8; 3] = [0, 100, 0];

struct TextureBitmap {
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

fn main() {
    if SCALE == 0 {
        panic!();
    }
    let apt = Apt::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let mut gfx = Gfx::new().unwrap();
    let mut old_keys = KeyPad::empty();
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

    while apt.main_loop() {
        hid.scan_input();
        if hid.keys_down().contains(KeyPad::START) {
            break;
        }
        let keys = hid.keys_held();
        if keys.contains(KeyPad::DPAD_LEFT) {
            player.angle -= 2;
            if player.angle <= 0 {
                player.angle = 359;
            }
            {
                let mut top_screen = gfx.top_screen.get_mut();
                top_screen.flush_buffers();
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
                let mut top_screen = gfx.top_screen.get_mut();
                top_screen.flush_buffers();
                let frame_buffer = top_screen.raw_framebuffer();
                (player, ray_cast) = ray_casting(player, ray_cast, map, &frame_buffer);
                top_screen.flush_buffers();
                top_screen.swap_buffers();
            }
        } else if keys.contains(KeyPad::DPAD_UP) {
            let angle = player.angle as f32;
            let pcos = 0.04 * deg_to_rad(&angle).cos();
            let psin = 0.04 * deg_to_rad(&angle).sin();
            let new_x = player.x + pcos;
            let new_y = player.y + psin;
            let check_x = (new_x + pcos * 20.0) as usize;
            let check_y = (new_y + psin * 20.0) as usize;
            if map[check_y][player.x as usize] == 0 {
                player.y = new_y;
            }
            if map[player.y as usize][check_x] == 0 {
                player.x = new_x;
            }
            /*
            if map[new_y as usize][new_x as usize] == 0 {
                player.x = new_x;
                player.y = new_y;
            }*/
            //player.x += 0.02 * deg_to_rad(&angle).cos();
            //player.y += 0.02 * deg_to_rad(&angle).sin();
            {
                let mut top_screen = gfx.top_screen.get_mut();
                top_screen.flush_buffers();
                let frame_buffer = top_screen.raw_framebuffer();
                (player, ray_cast) = ray_casting(player, ray_cast, map, &frame_buffer);
                top_screen.flush_buffers();
                top_screen.swap_buffers();
            }
        } else if keys.contains(KeyPad::DPAD_DOWN) {
            let angle = player.angle as f32;
            let pcos = 0.04 * deg_to_rad(&angle).cos();
            let psin = 0.04 * deg_to_rad(&angle).sin();
            if map[(psin + player.y) as usize][(pcos + player.x) as usize] == 0 {
                player.x -= pcos;
                player.y -= psin;
            }
            //player.x += 0.02 * deg_to_rad(&angle).cos();
            //player.y += 0.02 * deg_to_rad(&angle).sin();
            {
                let mut top_screen = gfx.top_screen.get_mut();
                top_screen.flush_buffers();
                let frame_buffer = top_screen.raw_framebuffer();
                (player, ray_cast) = ray_casting(player, ray_cast, map, &frame_buffer);
                top_screen.flush_buffers();
                top_screen.swap_buffers();
            }
        }
        old_keys = keys;
        gfx.wait_for_vblank();
    }
}

fn deg_to_rad(deg: &f32) -> f32 {
    return (*deg * PI / 180.0);
}

fn draw_filled_rec(
    frame_buffer: &RawFrameBuffer<'_>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: &[u8; 3],
    texture: bool,
    texture2: bool,
    texture_slice: Vec<u8>,
    texture_slice_2: &[u8],
) {
    let frame_buffer_slice = unsafe {
        std::slice::from_raw_parts_mut(
            frame_buffer.ptr,
            ((frame_buffer.height * frame_buffer.width) * 3) as usize,
        )
    };
    let mut step: f32 = 0.0;
    let mut chosen_color = color;
    let mut prev_index: usize = 0;
    let mut inc: u32 = 0;
    if texture {
        //step = height as f32 / GREY_BRICK.size as f32;
        if height > 240 {
            if height % 2 == 0 {
                inc = (height - 240) / 2;
            } else {
                inc = (height - 239) / 2;
            }
        }
        step = min(height, 240) as f32 / GREY_BRICK.size as f32;
    }
    for i in 0..min(height, 240) {
        if texture {
            if i > 0 {
                let x = ((i + inc) as f32 / step) as usize;
                if x > prev_index {
                    prev_index = x;
                    let max: usize;
                    if inc > 0 {
                        max = 7 - (inc as f32 / step) as usize;
                    }
                    if prev_index > 7 {
                        prev_index = 7;
                    }
                }
            }
            chosen_color = &GREY_BRICK.colors[texture_slice[prev_index] as usize];
        }
        for a in 0..width {
            let new_x = x + a;
            let new_y = y + i;
            if texture2 {
                chosen_color = &GREY_BRICK.colors[texture_slice_2[i as usize] as usize];
            }
            if new_x < frame_buffer.height as u32 && new_y < frame_buffer.width as u32 {
                let pixel_index = ((new_x) * frame_buffer.width as u32 + (new_y)) as usize * 3;
                frame_buffer_slice[pixel_index..pixel_index + 3].copy_from_slice(chosen_color);
            } else {
                println!(
                    "Invalid coordinates: ({}, {}) for buffer: {:#?}",
                    new_x, new_y, frame_buffer
                );
                let mut file = File::create("output.txt").expect("Failed to create output.txt");
                let time = Instant::now();
                writeln!(
                    file,
                    "{new_x} {new_y}         {:#?} {} time:{:?}",
                    frame_buffer,
                    frame_buffer_slice.len(),
                    time
                )
                .expect("Failed to write to file");
            }
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
        //if distance <= 1.0 {
        //    distance = 1.0;
        //}
        let wall_height = 120.0 / distance;
        let half_wall_height = wall_height / 2.0;
        let mut wall_start = 120 - wall_height as u32;
        let mut roof_start = 120 + wall_height as u32;
        if distance <= 1.0 {
            wall_start = 1;
            roof_start = 240;
        }
        let texture_pos_x =
            ((GREY_BRICK.size * (ray_struct.x + ray_struct.y) as u8) % GREY_BRICK.size) as usize;
        let texture_slice = get_texture_slice(texture_pos_x);
        let true_height = wall_height as u32 * 2;
        let v_tex_vec = get_vert_tex_map(&texture_slice, &true_height);
        let v_tex_slice: &[u8];
        if v_tex_vec.len() <= 240 {
            v_tex_slice = &v_tex_vec[..];
        } else {
            let overflow = v_tex_vec.len() - 240;
            let half_overflow: usize;
            if overflow % 2 == 0 {
                half_overflow = overflow / 2;
            } else {
                half_overflow = (overflow + 1) / 2;
            }
            v_tex_slice = &v_tex_vec[half_overflow..(v_tex_vec.len() - half_overflow)];
        }
        let empty_vec3: Vec<u8> = vec![];
        draw_filled_rec(
            frame_buffer,
            i as u32 * SCALE,
            wall_start - 1,
            SCALE,
            wall_height as u32 * 2,
            &SQUARE_COLOR,
            false,
            true,
            empty_vec3,
            v_tex_slice,
        );

        if wall_height < 120.0 {
            let empty_vec: Vec<u8> = vec![];
            let empty_vec2: Vec<u8> = vec![];
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
                false,
                empty_vec,
                empty_slice,
            );
            draw_filled_rec(
                frame_buffer,
                i as u32 * SCALE,
                (wall_start + (wall_height as u32 * 2)),
                SCALE,
                SCREEN_HEIGHT as u32 - (wall_start + (wall_height as u32 * 2)),
                &SQUARE_COLOR2,
                false,
                false,
                empty_vec2,
                empty_slice2,
            );
        }
    }
    return (player, ray);
}

fn get_texture_slice(texture_pos_x: usize) -> Vec<u8> {
    let mut tslice: Vec<u8> = Vec::new();
    for i in GREY_BRICK.bit_map.iter() {
        tslice.push(i[texture_pos_x]);
    }
    tslice
}

fn get_vert_tex_map<'a>(t_slice: &'a Vec<u8>, height: &'a u32) -> Vec<u8> {
    let mut v_map: Vec<u8> = Vec::with_capacity(*height as usize);
    let mut prev_index = 0;
    let step = *height as f32 / 8.0;
    for i in (1..height + 1) {
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
    /*
    if v_map.len() <= 240 {
        //let v_slice: &[u8] = &v_map[..];
        return v_map;
    }
    let overflow = v_map.len() - 240;
    let mut half_overflow: usize;
    if overflow % 2 == 0 {
        half_overflow = overflow / 2;
    } else {
        half_overflow = (overflow + 1) / 2;
    }
    let v_slice: &[u8] = v_map[half_overflow..(*height as usize - half_overflow)];
    return &v_slice;
    */
}
