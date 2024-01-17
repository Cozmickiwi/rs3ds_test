use std::borrow::BorrowMut;
use std::f32::consts::PI;
use std::fs::File;
use std::io::{self, Write};
use std::time::SystemTime;
use std::time::{Duration, Instant};

use ctru::{
    prelude::*,
    services::gfx::{self, Flush, RawFrameBuffer, Screen, Swap, TopScreen, TopScreenLeft},
};
use nalgebra::{matrix, Matrix};

struct RayCasting {
    increment_angle: f32,
    precision: u8,
}

struct Player {
    fov: u8,
    half_fov: u8,
    x: f32,
    y: f32,
    angle: u8,
}

const SCREEN_WIDTH: u16 = 400;
const SCREEN_HEIGHT: u16 = 240;

const SQUARE_COLOR_R: u8 = 255;
const SQUARE_COLOR_G: u8 = 0;
const SQUARE_COLOR_B: u8 = 0;

pub static SQUARE_COLOR: [u8; 3] = [SQUARE_COLOR_B, SQUARE_COLOR_G, SQUARE_COLOR_R];

fn main() {
    let apt = Apt::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let mut gfx = Gfx::new().unwrap();
    let mut top_screen = gfx.top_screen.get_mut();
    top_screen.swap_buffers();
    let frame_buffer = top_screen.raw_framebuffer();
    //let mut old_keys = KeyPad::empty();
    //draw_filled_rec(frame_buffer, 300, 100, 50, 15);

    let player = Player {
        fov: 60,
        half_fov: 30,
        x: 2.0,
        y: 2.0,
        angle: 45,
    };

    let ray_cast = RayCasting {
        increment_angle: player.fov as f32 / frame_buffer.width as f32,
        precision: 64,
    };

    /*let map = matrix![
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1;
        1, 0, 0, 0, 0, 0, 0, 0, 0, 1;
        1, 0, 0, 0, 0, 0, 0, 0, 0, 1;
        1, 0, 0, 1, 1, 0, 1, 0, 0, 1;
        1, 0, 0, 1, 0, 0, 1, 0, 0, 1;
        1, 0, 0, 1, 0, 0, 1, 0, 0, 1;
        1, 0, 0, 1, 0, 1, 1, 0, 0, 1;
        1, 0, 0, 0, 0, 0, 0, 0, 0, 1;
        1, 0, 0, 0, 0, 0, 0, 0, 0, 1;
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1
    ];

    for i in 0..10 {
        draw_filled_rec(
            &frame_buffer,
            100 + (i * 10 as u32),
            100,
            30,
            30,
            &SQUARE_COLOR,
        );
    }*/
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

    //   let map: Matrix<u8, 10, 10, _> = Matrix::from_row_slice(&map_data);*/
    ray_casting(player, ray_cast, map, &frame_buffer);
    top_screen.flush_buffers();
    top_screen.swap_buffers();
    while apt.main_loop() {
        hid.scan_input();
        if hid.keys_down().contains(KeyPad::START) {
            break;
        }
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
    color: &[u8],
) {
    let frame_buffer_slice = unsafe {
        std::slice::from_raw_parts_mut(
            frame_buffer.ptr,
            ((frame_buffer.height * frame_buffer.width) * 3) as usize,
        )
    };
    println!(
        "Buffer dimensions: {} x {}",
        frame_buffer.width, frame_buffer.height
    );
    for i in 0..height {
        for a in 0..width {
            let new_x = x + a;
            let new_y = y + i;
            if new_x < frame_buffer.height as u32 && new_y < frame_buffer.width as u32 {
                let pixel_index = ((new_x) * frame_buffer.width as u32 + (new_y)) as usize * 3;
                frame_buffer_slice[pixel_index..pixel_index + 3].copy_from_slice(&SQUARE_COLOR);
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
) {
    let mut ray_angle = player.angle as f32 - player.half_fov as f32;
    for i in 1..SCREEN_WIDTH {
        let mut ray_struct = Ray {
            x: player.x,
            y: player.y,
        };
        //increment
        ray_angle += ray.increment_angle;
        //ray_angle += 1;
        let ray_rad = deg_to_rad(&ray_angle);
        let pres = ray.precision as f32;
        let ray_cos = ray_rad.cos() / pres;
        let ray_sin = ray_rad.sin() / pres;
        // wall check
        let mut wall = 0;
        while wall == 0 {
            ray_struct.x += ray_cos;
            ray_struct.y += ray_sin;
            //wall = map.index((ray_struct.y as u16, ray_struct.x as u16));
            //            wall = (map[ray_struct.x as usize])[ray_struct.y as usize];
            wall = map[ray_struct.y as usize][ray_struct.x as usize];
        }
        let distance = ((player.x as f32 - ray_struct.x as f32).powi(2)
            + (player.y as f32 - ray_struct.y as f32).powi(2))
        .sqrt();
        // wall height
        let wall_height = 120.0 / distance;
        let half_wall_height = wall_height / 2.0;
        let wall_start = 120 - wall_height as u32;
        let roof_start = 120 + wall_height as u32;

        draw_filled_rec(
            frame_buffer,
            i.into(),
            wall_start,
            1,
            wall_height as u32 * 2,
            &SQUARE_COLOR,
        );
    }
}
