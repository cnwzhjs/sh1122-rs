pub use super::ifce::{ Sh1122Interface, Framebuffer };
use super::prelude::*;
use super::cmds;
use std::cmp::min;

const PIXEL_BITS: usize = 4;
const PIXEL_SHIFT: usize = 8 - PIXEL_BITS;
const PIXEL_MASK: u8 = 0xf;

pub struct Sh1122Device<T: Sh1122Interface> {
    interface: T,
    width: usize,
    height: usize,
    buf: Vec<u8>
}

impl<T: Sh1122Interface> Sh1122Device<T> {
    pub fn with_interface(interface: T, width: usize, height: usize) -> Self {
        Self {
            interface,
            width,
            height,
            buf: vec![0; width * height * PIXEL_BITS / 8]
        }
    }

    pub fn display_on(&mut self) -> Result<()> {
        self.interface.write_cmd(cmds::SET_DISP | 0x01u8, &[])
    }

    pub fn display_off(&mut self) -> Result<()> {
        self.interface.write_cmd(cmds::SET_DISP | 0x00u8, &[])
    }

    fn set_col_adr(&mut self, col: u8) -> Result<()> {
        self.interface.write_cmd(cmds::SET_COL_ADR_LSB | (col & 0x0fu8), &[])?;
        self.interface.write_cmd(cmds::SET_COL_ADR_MSB | ((col >> 4) & 0x0fu8), &[])?;
        Ok(())
    }

    fn set_row_adr(&mut self, row: u8) -> Result<()> {
        self.interface.write_cmd(cmds::SET_ROW_ADR, &[row])?;
        Ok(())
    }

    fn set_start_line(&mut self, line: u8) -> Result<()> {
        self.interface.write_cmd(cmds::SET_DISP_START_LINE | (line & 0x3fu8), &[])?;
        Ok(())
    }

    fn set_seg_remap_off(&mut self) -> Result<()> {
        self.interface.write_cmd(cmds::SET_SEG_REMAP | 0x00u8, &[])?;
        Ok(())
    }

    fn set_seg_remap_on(&mut self) -> Result<()> {
        self.interface.write_cmd(cmds::SET_SEG_REMAP | 0x01u8, &[])?;
        Ok(())
    }

    fn set_mux_ratio(&mut self, ratio: u8) -> Result<()> {
        self.interface.write_cmd(cmds::SET_MUX_RATIO, &[ratio])?;
        Ok(())
    }

    fn set_com_output_scan_dir(&mut self, dir: u8) -> Result<()> {
        self.interface.write_cmd(cmds::SET_COM_OUT_DIR | (dir & 0x01u8), &[])?;
        Ok(())
    }

    fn set_display_offset(&mut self, offset: u8) -> Result<()> {
        self.interface.write_cmd(cmds::SET_DISP_OFFSET, &[offset])?;
        Ok(())
    }

    pub fn set_contrast(&mut self, contrast: u8) -> Result<()> {
        self.interface.write_cmd(cmds::SET_CONTRAST, &[contrast])?;
        Ok(())
    }

    fn set_entire_on(&mut self) -> Result<()> {
        self.interface.write_cmd(cmds::SET_ENTIRE_ON, &[])?;
        Ok(())
    }

    pub fn set_inverted(&mut self, inverted: bool) -> Result<()> {
        self.interface.write_cmd(cmds::SET_NORM_INV | if inverted { 1 } else { 0 }, &[])?;
        Ok(())
    }

    pub fn init_display(&mut self) -> Result<()> {
        self.display_off()?;
        self.set_row_adr(0)?;
        self.set_col_adr(0)?;
        self.set_start_line(0)?;
        self.set_seg_remap_on()?;
        self.set_seg_remap_off()?;
        self.set_mux_ratio((self.height - 1) as u8)?;
        self.set_com_output_scan_dir(0)?;
        self.set_display_offset(0)?;
        self.set_contrast(0x80u8)?;
        self.set_entire_on()?;
        self.set_inverted(false)?;
        self.display_on()?;

        self.flush()?;

        Ok(())
    }

    pub fn print_buffer(&self) {
        for i in 0..((self.buf.len() + 15) / 16) {
            let start = i * 16;
            let end = min(start + 16, self.buf.len());
            let chunk = &self.buf[start..end];
            println!("{:02X?}", chunk);
        }
    }

    pub fn flush(&mut self) -> Result<()> {
        self.set_col_adr(0)?;
        self.set_row_adr(0)?;
        self.interface.write_data(&self.buf)?;
        Ok(())
    }

    pub fn partial_flush(&mut self, x: usize, y: usize, width: usize, height: usize) -> Result<()> {
        if x >= self.width || y >= self.height {
            return Ok(());
        }

        let mut x = x;
        let mut width = width;
        let mut height = height;

        let (_, bit_idx) = self.get_idx(x, y);
        if bit_idx != 8 - PIXEL_BITS {
            // this is not aligned to pixel
            let adjust = (8 - PIXEL_BITS - bit_idx) / PIXEL_BITS;
            x = x - adjust;
            width = width + adjust;
        }

        width = min(width, self.width - x);
        height = min(height, self.height - y);

        // println!("partial flushing @ {}, {}, with {} x {}", x, y, width, height);

        let mut jobs = Vec::new();

        if width == self.width {
            jobs.push((x, y, width * height * PIXEL_BITS / 8));
        } else {
            for j in 0..height {
                jobs.push((x, y + j, (width * PIXEL_BITS + PIXEL_BITS) / 8));
            }
        }

        for (x, y, bytes) in jobs {
            let (byte_idx, _) = self.get_idx(x, y);
            self.set_col_adr(x as u8)?;
            self.set_row_adr(y as u8)?;
            self.interface.write_data(&self.buf[byte_idx..byte_idx + bytes])?;
        }

        Ok(())
    }

    fn get_idx(&self, x: usize, y: usize) -> (usize, usize) {
        if cfg!(debug_assertions) {
            assert!(x < self.width as usize);
            assert!(y < self.height as usize);
        }

        let idx = x + y * (self.width as usize);
        let byte_idx = idx * PIXEL_BITS / 8;
        let bit_idx = 8 - PIXEL_BITS - PIXEL_BITS * (idx - byte_idx * 8 / PIXEL_BITS);

        (byte_idx, bit_idx)
    }
}

impl<If: Sh1122Interface> Framebuffer<u8> for Sh1122Device<If> {
    fn get_width(&self) -> usize {
        self.width as usize
    }

    fn get_height(&self) -> usize {
        self.height as usize
    }

    fn get_pixel(&self, x: usize, y: usize) -> u8 {
        let (byte_idx, bit_idx) = self.get_idx(x, y);
        let byte = self.buf[byte_idx];
        let dat = (byte & (PIXEL_MASK << bit_idx)) >> bit_idx;

        dat << PIXEL_SHIFT
    }

    fn set_pixel(&mut self, x: usize, y: usize, pixel: u8) {
        let (byte_idx, bit_idx) = self.get_idx(x, y);
        let mask = PIXEL_MASK << bit_idx;
        let byte = (self.buf[byte_idx] & !mask) | ((pixel >> PIXEL_SHIFT) << bit_idx);
        // println!("x: {}, y: {}, idx({}, {}), pixel: {}, mask: {}, byte: {:02x} => {:02x}", x, y, byte_idx, bit_idx, pixel, mask, self.buf[byte_idx], byte);
        self.buf[byte_idx] = byte;
    }

    fn fill(&mut self, pixel: u8) {
        let pixel = pixel >> PIXEL_SHIFT;
        let mut dat = 0u8;
        for i in 0..(8 / PIXEL_BITS) {
            dat = dat | (pixel << (i * PIXEL_BITS));
        }
        self.buf.fill(dat);
    }

    fn partial_fill(&mut self, x: usize, y: usize, width: usize, height: usize, pixel: u8) {
        for j in 0..height {
            for i in 0..width {
                self.set_pixel(x + i, y + j, pixel);
            }
        }
    }
}
