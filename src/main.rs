use ctru::{prelude::*, services::gfx::{Screen, self, Swap, RawFrameBuffer}};

fn main() {
    let apt = Apt::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let gfx = Gfx::new().unwrap();
    let console = Console::new(gfx.top_screen.borrow_mut());
    //println!("Hello, World!");
    //println!("\x1b[29;16HPress Start to exit");
    let mut bottom_screen = gfx.bottom_screen.borrow_mut();
    bottom_screen.set_double_buffering(false);
    bottom_screen.swap_buffers();

    let frame_buffer = bottom_screen.raw_framebuffer();
    let mut old_keys = KeyPad::empty();
    draw_filled_square(frame_buffer, 100, 100, 100);

    while apt.main_loop() {
        hid.scan_input();
        if hid.keys_down().contains(KeyPad::START) {
            break;
        }
        let keys = hid.keys_held();
        /*if keys != old_keys {
            console.clear();
            println!("\x1b[29;16HPress Start to exit");
            println!("\x1b[3;0H");
            println!("{:#?}", keys);
            if hid.keys_down().contains(KeyPad::START) {
                break;
            }
        }
        old_keys = keys;
        */gfx.wait_for_vblank();
    }
}
const SQUARE_COLOR_R: u8 = 255;
const SQUARE_COLOR_G: u8 = 0;
const SQUARE_COLOR_B: u8 = 0;


static SQUARE_COLOR: [u8; 3] = [SQUARE_COLOR_B, SQUARE_COLOR_G, SQUARE_COLOR_R];
fn draw_filled_square(frame_buffer: RawFrameBuffer<'_>, x: u16, y: u16, size: u16) {
    let frame_buffer_slice = unsafe { std::slice::from_raw_parts_mut(frame_buffer.ptr, (frame_buffer.width * frame_buffer.width * 3) as usize) };
    for i in 0..size {
        for a in 0..size {
            let pixel_index = ((x + i) * frame_buffer.width as u16 + (y + a)) as usize * 3;
            unsafe {
                frame_buffer_slice[pixel_index..pixel_index + 3].copy_from_slice(&SQUARE_COLOR);
            }
            //arr.push(pixel_index);

        }
    }
}
