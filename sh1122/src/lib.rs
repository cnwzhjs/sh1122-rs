#![allow(dead_code)]

mod ifce;
mod dev;
mod cmds;
mod prelude;

pub use prelude::*;
pub use ifce::{ Sh1122Interface, Framebuffer };
pub use dev::Sh1122Device;
