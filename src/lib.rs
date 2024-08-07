#![no_std]

use embedded_hal::blocking::i2c::{Write, WriteRead};

const MAX17048_ADDR: u8 = 0x6C;
const DEFAULT_RCOMP: u8 = 0x97;

pub struct Max17048<I> {
    i2c: I,
    addr: u8,
    recv_buffer: [u8; 2],
}

impl<I, E> Max17048<I>
where
    I: WriteRead<Error = E> + Write<Error = E>,
    E: core::fmt::Debug,
{
    pub fn new(i2c: I, addr: u8) -> Self {
        let mut max = Max17048 {
            i2c,
            addr,
            recv_buffer: [0u8; 2],
        };
        max
    }

    pub fn new_with_default_addr(i2c: I) -> Self {
        Self::new(i2c, MAX17048_ADDR)
    }

    pub fn default_compensation(&mut self) -> Result<(), E> {
        self.compensation(DEFAULT_RCOMP)
    }

    pub fn version(&mut self) -> Result<u16, E> {
        self.read(0x08)
    }

    pub fn soc(&mut self) -> Result<u16, E> {
        match self.read(0x04) {
            Ok(val) => Ok(val / 256),
            Err(e) => Err(e),
        }
    }

    /// Return C/Rate in %/hr
    pub fn charge_rate(&mut self) -> Result<f32, E> {
        match self.read(0x16) {
            Ok(val) => Ok(val as f32 * 0.208),
            Err(e) => Err(e),
        }
    }

    pub fn vcell(&mut self) -> Result<f32, E> {
        match self.read(0x02) {
            Ok(val) => Ok(val as f32 * 0.000078125),
            Err(e) => Err(e),
        }
    }

    pub fn temp_compensation(&mut self, temp: f32) -> Result<(), E> {
        let rcomp = if temp > 20.0 {
            DEFAULT_RCOMP as f32 + (temp - 20.0) * -0.5
        } else {
            DEFAULT_RCOMP as f32 + (temp - 20.0) * -5.0
        };
        self.compensation(rcomp as u8)
    }

    pub fn release(self) -> I {
        self.i2c
    }

    fn compensation(&mut self, rcomp: u8) -> Result<(), E> {
        // read the current reg vals
        match self.read(0x0C) {
            Ok(mut value) => {
                value &= 0x00FF;
                value |= (rcomp as u16) << 8;
                // write to the rcomp bits only
                self.write(0x0C, value)?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn read(&mut self, reg: u8) -> Result<u16, E> {
        match self
            .i2c
            .write_read(self.addr, &[reg], &mut self.recv_buffer)
        {
            Ok(_) => Ok((self.recv_buffer[0] as u16) << 8 | self.recv_buffer[1] as u16),
            Err(e) => Err(e),
        }
    }

    fn write(&mut self, reg: u8, value: u16) -> Result<(), E> {
        self.i2c.write(self.addr, &[reg])?;
        let msb = ((value & 0xFF00) >> 8) as u8;
        let lsb = ((value & 0x00FF) >> 0) as u8;
        self.i2c.write(self.addr, &[msb, lsb])?;
        Ok(())
    }
}
