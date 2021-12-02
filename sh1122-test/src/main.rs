use freetype::face::LoadFlag;
use rppal::i2c::I2c;
use sh1122_rppal::Sh1122I2cInterface;
use sh1122::*;
use std::cmp::max;

fn draw_bitmap<T: Framebuffer<u8>>(bitmap: &freetype::Bitmap, display: &mut T, x: i32, y: i32, color: u8) {
    let buf = bitmap.buffer();
    let cols = bitmap.width();
    let rows = bitmap.rows();
    let pitch = bitmap.pitch();

    for row in 0..rows {
        for col in 0..cols {
            let idx = (col + row * pitch) as usize;
            let px = x + col;
            let py = y + row;

            if px < 0 || py < 0 || px >= display.get_width() as i32 || py >= display.get_height() as i32 {
                continue;
            }

            let alpha = buf[idx];
            if alpha == 0 {
                continue;
            }

            let pixel = if alpha == 255 {
                color
            } else {
                let origin = display.get_pixel(px as usize, py as usize) as u16;
                let alpha = alpha as u16;
                let color = color as u16;
                ((origin * (255 - alpha) + color * alpha) / 255) as u8
            };

            display.set_pixel(px as usize, py as usize, pixel);
        }
    }
}

fn put_char<T: Framebuffer<u8>>(face: &freetype::Face, display: &mut T, c: char, x: usize, y: usize, color: u8) -> Result<usize> {
    face.load_char(c as usize, LoadFlag::RENDER)?;
    let glyph = face.glyph();
    let bitmap = glyph.bitmap();

    draw_bitmap(&bitmap, display, x as i32 + glyph.bitmap_left(), y as i32 - glyph.bitmap_top(), color);

    Ok(x + ((glyph.advance().x >> 6) as usize))
}

fn draw_text<T: Framebuffer<u8>>(face: &freetype::Face, display: &mut T, text: &str, x: usize, y: usize, color: u8) -> Result<usize> {
    let mut dx = x;
    for c in text.chars() {
        dx = put_char(face, display, c, dx, y, color)?;
    }

    Ok(dx)
}

fn show_system_info<T: Framebuffer<u8>>(face: &freetype::Face, info: &rppal::system::DeviceInfo, display: &mut T) -> Result<(usize, usize)> {
    let mut end_x = draw_text(face, display, format!("Model: {}", info.model()).as_str(), 0, 12, 0xffu8)?;
    end_x = max(end_x, draw_text(face, display, format!("CPU: {}", info.soc()).as_str(), 0, 32, 0x3fu8)?);

    Ok((end_x, 32))
}

fn main() {
    println!("open i2c device");
    let mut i2c = I2c::with_bus(1).unwrap();
    println!("construct sh1122 if");
    let sh1122if = Sh1122I2cInterface::new(&mut i2c, 0x3cu8).unwrap();
    println!("construct sh1122");
    let mut sh1122 = Sh1122Device::with_interface( sh1122if, 256, 64);
    println!("init display");
    sh1122.init_display().unwrap();
    println!("fill with 64");
    sh1122.fill(0);
    println!("fetching device info");
    let system_info = rppal::system::DeviceInfo::new().unwrap();
    println!("displaying system info");
    let lib = freetype::Library::init().unwrap();
    let face = lib.new_face("/usr/share/fonts/truetype/roboto/unhinted/RobotoTTF/Roboto-Regular.ttf", 0).unwrap();
    face.set_char_size(0, 12 * 64, 0, 72).unwrap();

    let (end_x, end_y) = show_system_info(&face, &system_info, &mut sh1122).unwrap();
    println!("flush to display");
    sh1122.partial_flush(0, 0, end_x, end_y).unwrap();

    let mut last_endx = sh1122.get_width() - 1;
    sh1122.partial_fill(0, 38, last_endx + 1, 20, 0x10);

    loop {
        let now = chrono::prelude::Local::now();
        sh1122.partial_fill(0, 38, last_endx + 1, 20, 0x3f);
        let end_x = draw_text(&face, &mut sh1122, now.to_rfc2822().as_str(), 0, 52, 0xffu8).unwrap();
        sh1122.partial_flush(0, 38, max(last_endx, end_x) + 1, 20).unwrap();
        last_endx = end_x;
        // std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
