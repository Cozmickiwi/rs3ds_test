use ctru::{prelude::*, services::gfx::Screen};

fn main() {
    let apt = Apt::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let gfx = Gfx::new().unwrap();
    let console = Console::new(gfx.top_screen.borrow_mut());
    //println!("Hello, World!");
    //println!("\x1b[29;16HPress Start to exit");

    let mut old_keys = KeyPad::empty();
    while apt.main_loop() {
        hid.scan_input();
        let keys = hid.keys_held();
        if keys != old_keys {
            console.clear();
            println!("\x1b[29;16HPress Start to exit");
            println!("\x1b[3;0H");
            println!("{:#?}", keys);
            if hid.keys_down().contains(KeyPad::START) {
                break;
            }
        }
        old_keys = keys;
        gfx.wait_for_vblank();
    }
}
