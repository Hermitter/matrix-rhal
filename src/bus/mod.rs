pub mod memory_map;
use crate::{error::Error, Device};
use memory_map::*;
extern crate spidev;
use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use nix::unistd::close;
use nix::{ioctl_read_bad, ioctl_write_ptr_bad};

//io!, ior!, iow!, and iorw! has been renamed to request_code_none!, request_code_read!, request_code_write!, and request_code_readwrite! respectively.
use nix::{request_code_read, request_code_write};

// spi imports
use spidev::{SpiModeFlags, Spidev, SpidevOptions, SpidevTransfer};
use std::io;
use std::io::prelude::*;

ioctl_read_bad!(ioctl_read, ioctl_code::READ, [u8]);
ioctl_write_ptr_bad!(ioctl_write, ioctl_code::WRITE, [u8]);

/// Bridge for talking to the MATRIX Kernel Modules.
/// Most, if not all, MATRIX functionality requires this Bus to read and write data.
#[derive(Debug)]
pub struct Bus {
    /// Path for the device file being used. This is what's used to communicate with the MATRIX Kernel.
    pub device_file: &'static str,
    /// File descriptor for kernel abstraction.
    pub regmap_fd: std::os::unix::io::RawFd,
    /// Type of MATRIX device that's currently attached.
    pub device_name: Device,
    /// The version of the board.
    pub device_version: u32,
    /// Number of LEDS on the MATRIX device.
    pub device_leds: u8,
    /// Frequency of the FPGA on the MATRIX device.
    pub fpga_frequency: u32,

    // SPI params
    spi_fd: i32,
    spi_mode: u32,
    spi_bits: u32,
    spi_speed: u32,
    spi_delay: u32,
}

impl Bus {
    /// Create, initialize, and return a MATRIX Bus
    pub fn init() -> Result<Bus, Error> {
        let mut bus = Bus {
            device_file: "/dev/spidev0.0",
            regmap_fd: 0,
            device_name: Device::Unknown,
            device_version: 0,
            device_leds: 0,
            fpga_frequency: 0,
            spi_fd: 0,
            spi_mode: 3,
            spi_bits: 8,
            spi_speed: 15000000,
            spi_delay: 0,
        };

        // open the file descriptor to communicate with the MATRIX kernel
        bus.regmap_fd = open(bus.device_file, OFlag::O_RDWR, Mode::empty())?;

        // - All SPI Magic Numbers
        const SPI_IOC_MAGIC: u8 = b'k'; // Defined in linux/spi/spidev.h

        // * Read Numbers
        let spi_ioc_rd_mode = request_code_read!(SPI_IOC_MAGIC, 1, 1);
        assert_eq!(spi_ioc_rd_mode, 2147576577);

        let spi_ioc_rd_bits_per_word = request_code_read!(SPI_IOC_MAGIC, 3, 1);
        assert_eq!(spi_ioc_rd_bits_per_word, 2147576579);

        let spi_ioc_rd_max_speed_hz = request_code_read!(SPI_IOC_MAGIC, 4, 4);
        assert_eq!(spi_ioc_rd_max_speed_hz, 2147773188);

        // * Write Numbers
        let spi_ioc_wr_mode = request_code_write!(SPI_IOC_MAGIC, 1, 1);
        assert_eq!(spi_ioc_wr_mode, 1073834753);

        let spi_ioc_wr_bits_per_word = request_code_write!(SPI_IOC_MAGIC, 3, 1);
        assert_eq!(spi_ioc_wr_bits_per_word, 1073834755);

        let spi_ioc_wr_max_speed_hz = request_code_write!(SPI_IOC_MAGIC, 4, 4);
        assert_eq!(spi_ioc_wr_max_speed_hz, 1074031364);

        // remove after testing
        println!("-------------------------------------------\n");
        Err(Error::InvalidGpioPin)
    }

    pub fn write(&self, write_buffer: &mut [u8]) {}

    pub fn read(&self, read_buffer: &mut [u8]) {}

    /// Close the file descriptor that's communicating with the MATRIX Kernel's device file.
    pub fn close(&self) {}

    /// Return the type of MATRIX device being used and the version of the board.
    fn get_device_info(&self) -> Result<(Device, u32), Error> {
        // create read buffer
        let mut data: [i32; 4] = [0; 4];
        data[0] = fpga_address::CONF as i32;
        data[1] = 8; // device_name(4 bytes) device_version(4 bytes)

        self.read(unsafe { std::mem::transmute::<&mut [i32], &mut [u8]>(&mut data) });
        let device_name = data[2];
        let device_version = data[3];

        Ok((
            match device_name {
                device_info::MATRIX_CREATOR => Device::Creator,
                device_info::MATRIX_VOICE => Device::Voice,
                _ => return Err(Error::UnknownDevice),
            },
            device_version as u32,
        ))
    }

    /// Updates the Bus to have the last known FPGA frequency of the MATRIX device.
    fn get_fpga_frequency(&self) -> Result<u32, Error> {
        // create read buffer
        let mut data: [i32; 3] = [0; 3];
        data[0] = (fpga_address::CONF + 4) as i32;
        data[1] = 4; // value0(2 bytes) value1(2bytes) // TODO: ask what these values represent

        self.read(unsafe { std::mem::transmute::<&mut [i32], &mut [u8]>(&mut data) });

        // extract both u16 numbers from u32
        let value0 = data[2] >> 16; // store 1st 16 bits
        let value1 = !(value0 << 16) & data[2]; // store 2nd 16 bits
        let frequency = (device_info::FPGA_CLOCK * value0 as u32) / value1 as u32;

        Ok(frequency)
    }
}
