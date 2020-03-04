// TODOs For Tomorrow
// TODO: Change all `update_pin_map` function to be fixed like `Mode`'s implementation.
// TODO: Stop returning a tuple for generate values since the history is a global object in `Gpio`.

use super::Gpio;
use crate::error::Error;

pub trait PinConfig {
    /// Returns a tuple with a number, binary representation of each pin config, and an FPGA address offset for the config being changed.
    fn update_pin_map(&self, pin: u8, gpio: &Gpio) -> Result<(u32, u16), Error>;
}

/// Specifies if a pin is being used for Output or Input signals.
#[derive(Copy, Clone)]
pub enum Mode {
    Input = 0,
    Output = 1,
}
impl PinConfig for Mode {
    fn update_pin_map(&self, pin: u8, gpio: &Gpio) -> Result<(u32, u16), Error> {
        let mode = *self as u16;
        let mask = 1 << pin;
        let mut pin_map = gpio.mode_pin_map.lock()?;

        *pin_map = mode << pin | (*pin_map & !mask);

        println!("mode: {:b}", *pin_map);
        Ok((*pin_map as u32, 0))
    }
}

// Specifies the current signal state of a pin.
#[derive(Copy, Clone)]
pub enum State {
    Off = 0,
    On = 1,
}

impl PinConfig for State {
    fn update_pin_map(&self, pin: u8, gpio: &Gpio) -> Result<(u32, u16), Error> {
        let state = *self as u16;
        let mask = 1 << pin;
        let mut pin_map = gpio.state_pin_map.lock()?;

        *pin_map = state << pin | (*pin_map & !mask);

        println!("state: {:b}", *pin_map);
        Ok((*pin_map as u32, 1))
    }
}

/// Specifies which function a pin is using.
#[derive(Copy, Clone)]
pub enum Function {
    Digital = 0,
    Pwm = 1,
}

impl PinConfig for Function {
    fn update_pin_map(&self, pin: u8, gpio: &Gpio) -> Result<(u32, u16), Error> {
        let function = *self as u16;
        let mask = 1 << pin;
        let mut pin_map = gpio.function_pin_map.lock()?;

        *pin_map = function << pin | (*pin_map & !mask);

        println!("function: {:b}", *pin_map);
        Ok((*pin_map as u32, 2))
    }
}