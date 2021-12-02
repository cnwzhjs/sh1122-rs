pub use super::prelude::*;
use rppal::i2c::I2c;

pub struct Sh1122I2cInterface<'a> {
    i2c: &'a mut I2c,
}

impl<'a> Sh1122I2cInterface<'a> {
    pub fn new(i2c: &'a mut I2c, address: u8) -> Result<Self> {
        i2c.set_slave_address((address & 0x7f) as u16)?;
        Ok(Self { i2c })
    }
}

impl<'a> Sh1122Interface for Sh1122I2cInterface<'a> {
    fn write_cmd(&mut self, cmd: u8, data: &[u8]) -> Result<()> {
        let mut dat = vec![0x00, cmd];
        dat.extend_from_slice(data);
        self.i2c.write( dat.as_slice())?;
        Ok(())
    }

    fn write_data(&mut self, data: &[u8]) -> Result<()> {
        let mut dat = vec![0x40u8];
        dat.extend_from_slice(data);
        self.i2c.write(dat.as_slice())?;
        Ok(())
    }
}
