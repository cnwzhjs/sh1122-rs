use super::prelude::*;
use std::fmt::{Display, Debug};

pub trait Sh1122Interface {
    fn write_cmd(&mut self, cmd: u8, date: &[u8]) -> Result<()>;
    fn write_data(&mut self, data: &[u8]) -> Result<()>;
}

pub trait Framebuffer<T: Copy + Clone + Display + Debug> {
    fn get_width(&self) -> usize;
    fn get_height(&self) -> usize;
    fn get_pixel(&self, x: usize, y: usize) -> T;
    fn set_pixel(&mut self, x: usize, y: usize, pixel: T);
    fn fill(&mut self, pixel: T);
    fn partial_fill(&mut self, x: usize, y: usize, width: usize, height: usize, pixel: T);
}
