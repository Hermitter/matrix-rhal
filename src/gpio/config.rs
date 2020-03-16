use super::Gpio;
use crate::error::Error;
use core::sync::atomic::Ordering;

pub trait PinConfig {
    /// Returns a tuple of a number (binary representation of each pin config) and an FPGA address offset for the config being changed.
    /// This function will also update the relevant pin map in its `Gpio` instance.
    fn update_pin_map(&self, pin: u8, gpio: &Gpio) -> Result<(u16, u16), Error>;
}

/// Represents a pin being used for `Output` or `Input`.
#[derive(Debug, Copy, Clone)]
pub enum Mode {
    Input = 0,
    Output = 1,
}

impl PinConfig for Mode {
    fn update_pin_map(&self, pin: u8, gpio: &Gpio) -> Result<(u16, u16), Error> {
        let mut pin_map = gpio.mode_pin_map.load(Ordering::Acquire);
        set_pin_config(pin, *self as u16, &mut pin_map);
        gpio.mode_pin_map.store(pin_map, Ordering::Release);

        Ok((pin_map, 0))
    }
}

/// Represents a pin being `On` or `Off`.
#[derive(Debug, Copy, Clone)]
pub enum State {
    Off = 0,
    On = 1,
}

impl PinConfig for State {
    fn update_pin_map(&self, pin: u8, gpio: &Gpio) -> Result<(u16, u16), Error> {
        let mut pin_map = gpio.state_pin_map.load(Ordering::Acquire);
        set_pin_config(pin, *self as u16, &mut pin_map);
        gpio.state_pin_map.store(pin_map, Ordering::Release);

        Ok((pin_map, 1))
    }
}

/// Represents a pin being used for `Digital` or `Pwm`.
#[derive(Debug, Copy, Clone)]
pub enum Function {
    Digital = 0,
    Pwm = 1,
}

impl PinConfig for Function {
    fn update_pin_map(&self, pin: u8, gpio: &Gpio) -> Result<(u16, u16), Error> {
        let mut pin_map = gpio.function_pin_map.load(Ordering::Acquire);
        set_pin_config(pin, *self as u16, &mut pin_map);
        gpio.function_pin_map.store(pin_map, Ordering::Release);

        Ok((pin_map, 2))
    }
}

/// Flips the desired binary bit in a configuration's pin map. Each bit represents a pin.
/// # Visual Example
/// ```
/// state_pin_map = 000000000000000; // all pins are OFF
/// state_pin_map = 100000000000011; // pins 0,1, and 15 are ON
/// ```
///
/// # Code Explanation
/// ```
///     let mut pin_map = 32771; // ->10000000000000011
///
///     let pin = 15;
///     let config = 0;
///     let mask = 1 << pin; // -> 1000000000000000
///
///     let config = config << pin; // -> 0000000000000000
///     let configured_map = pin_map & !mask; // -> 0000000000000011
///     
///     // this operation is only relevant when a bit is being flipped ON
///     pin_map = config | configured_map; // -> 0000000000000011
/// ```
fn set_pin_config(pin: u8, config: u16, pin_map: &mut u16) {
    let mask = 1 << pin;
    *pin_map = config << pin | (*pin_map & !mask);
}
