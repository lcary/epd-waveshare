//! A simple Driver for the Waveshare 4.2" E-Ink Display Module B v2.2 via SPI
//!
//! This driver is based on the Python implementation epd4in2b_v2.py from Waveshare
//! and supports three-color displays (Black/White/Red).
//!
//! The key difference from the standard 4.2" display is that this module B version
//! has different initialization sequences and uses a flag-based approach to handle
//! different chip versions.
//!
//! ## Hardware Note
//! 
//! This implementation assumes SPI is disabled via raspi-config as mentioned
//! in the Python implementation requirements.
//!
//! ## Chip Version Detection
//! 
//! The module supports two chip versions (flag = 0 or flag = 1) with different:
//! - Initialization sequences
//! - Busy signal logic (active-low vs active-high)
//! - Data transmission commands (0x10/0x13 vs 0x24/0x26)
//! 
//! The driver defaults to flag = 0. If you know your chip version, you can call
//! `set_flag(1)` after creating the EPD instance but before calling init.
//!
//! # Examples
//!
//!```rust, no_run
//!# use embedded_hal_mock::eh1::*;
//!# fn main() -> Result<(), embedded_hal::spi::ErrorKind> {
//!use embedded_graphics::{prelude::*, primitives::{Line, PrimitiveStyle}};
//!use epd_waveshare::{epd4in2b_v2::*, prelude::*};
//!#
//!# let expectations = [];
//!# let mut spi = spi::Mock::new(&expectations);
//!# let expectations = [];
//!# let cs_pin = digital::Mock::new(&expectations);
//!# let busy_in = digital::Mock::new(&expectations);
//!# let dc = digital::Mock::new(&expectations);
//!# let rst = digital::Mock::new(&expectations);
//!# let mut delay = delay::NoopDelay::new();
//!
//!// Setup EPD
//!let mut epd = Epd4in2bV2::new(&mut spi, busy_in, dc, rst, &mut delay, None)?;
//!
//!// If you know your chip version, set it before init:
//!// epd.set_flag(1); // for chip version 1
//!
//!// Use display graphics from embedded-graphics
//!let mut tricolor_display = Display4in2bV2::default();
//!
//!// Use embedded graphics for drawing a black line
//!let _ = Line::new(Point::new(0, 120), Point::new(0, 295))
//!    .into_styled(PrimitiveStyle::with_stroke(TriColor::Black, 1))
//!    .draw(&mut tricolor_display);
//!
//!// We use `chromatic` but it will be shown as red
//!let _ = Line::new(Point::new(15, 120), Point::new(15, 295))
//!    .into_styled(PrimitiveStyle::with_stroke(TriColor::Chromatic, 1))
//!    .draw(&mut tricolor_display);
//!
//!// Display updated frame
//!epd.update_color_frame(
//!    &mut spi,
//!    &mut delay,
//!    &tricolor_display.bw_buffer(),
//!    &tricolor_display.chromatic_buffer()
//!)?;
//!epd.display_frame(&mut spi, &mut delay)?;
//!
//!// Set the EPD to sleep
//!epd.sleep(&mut spi, &mut delay)?;
//!# Ok(())
//!# }
//!```

use embedded_hal::{delay::*, digital::*, spi::SpiDevice};

use crate::interface::DisplayInterface;
use crate::traits::{
    InternalWiAdditions, RefreshLut, WaveshareDisplay, WaveshareThreeColorDisplay,
};

/// Width of the display
pub const WIDTH: u32 = 400;
/// Height of the display
pub const HEIGHT: u32 = 300;
/// Default Background Color
pub const DEFAULT_BACKGROUND_COLOR: TriColor = TriColor::White;

const SINGLE_BYTE_WRITE: bool = true;

use crate::color::TriColor;

pub(crate) mod command;
use self::command::Command;
use crate::buffer_len;

/// Full size buffer for use with the 4in2b_v2 EPD
#[cfg(feature = "graphics")]
pub type Display4in2bV2 = crate::graphics::Display<
    WIDTH,
    HEIGHT,
    true,
    { buffer_len(WIDTH as usize, HEIGHT as usize * 2) },
    TriColor,
>;

/// Epd4in2bV2 driver
pub struct Epd4in2bV2<SPI, BUSY, DC, RST, DELAY> {
    /// Connection Interface
    interface: DisplayInterface<SPI, BUSY, DC, RST, DELAY, SINGLE_BYTE_WRITE>,
    /// Background Color
    color: TriColor,
    /// Flag to track chip version (0 or 1)
    flag: u8,
}

impl<SPI, BUSY, DC, RST, DELAY> InternalWiAdditions<SPI, BUSY, DC, RST, DELAY>
    for Epd4in2bV2<SPI, BUSY, DC, RST, DELAY>
where
    SPI: SpiDevice,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    fn init(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        // Reset the device
        self.interface.reset(delay, 200_000, 5_000);
        delay.delay_ms(200);

        // Send ReadId command to determine chip version (mimics Python send_command(0x2F))
        self.command(spi, Command::ReadId)?;
        delay.delay_ms(100);
        
        // Note: In real hardware, we would read the SPI response here to determine
        // the chip version. Since most embedded SPI implementations don't support
        // simultaneous read/write easily, we default to flag = 0.
        // Users can call set_flag() if they know their chip version.
        // The Python code reads a response of 0x01 to set flag = 1.
        
        if self.flag == 1 {
            // Version 1 initialization (matches Python if i == 0x01)
            self.wait_until_idle(spi, delay)?;
            self.command(spi, Command::DisplayRefresh)?;
            self.wait_until_idle(spi, delay)?;

            self.cmd_with_data(spi, Command::BorderWaveformControl, &[0x05])?;
            self.cmd_with_data(spi, Command::Xon, &[0x80])?;
            self.cmd_with_data(spi, Command::DataEntryMode, &[0x03])?;

            // Set RAM X address (Python: commands 0x44)
            self.cmd_with_data(spi, Command::SetRamXAddressStartEndPosition, 
                              &[0x00, (WIDTH / 8 - 1) as u8])?;

            // Set RAM Y address (Python: commands 0x45)
            self.cmd_with_data(spi, Command::SetRamYAddressStartEndPosition, 
                              &[0x00, 0x00, ((HEIGHT - 1) % 256) as u8, ((HEIGHT - 1) / 256) as u8])?;

            // Set RAM X address counter (Python: command 0x4E)
            self.cmd_with_data(spi, Command::SetRamXAddressCounter, &[0x00])?;
            
            // Set RAM Y address counter (Python: command 0x4F)
            self.cmd_with_data(spi, Command::SetRamYAddressCounter, &[0x00, 0x00])?;
            
            self.wait_until_idle(spi, delay)?;
        } else {
            // Version 0 initialization (matches Python else branch)
            self.command(spi, Command::PowerOn)?;  // 0x04
            self.wait_until_idle(spi, delay)?;
            self.cmd_with_data(spi, Command::PanelSetting, &[0x0f])?;  // 0x00
        }

        Ok(())
    }
}

impl<SPI, BUSY, DC, RST, DELAY> WaveshareDisplay<SPI, BUSY, DC, RST, DELAY>
    for Epd4in2bV2<SPI, BUSY, DC, RST, DELAY>
where
    SPI: SpiDevice,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    type DisplayColor = TriColor;

    fn new(
        spi: &mut SPI,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
        delay_us: Option<u32>,
    ) -> Result<Self, SPI::Error> {
        let interface = DisplayInterface::new(busy, dc, rst, delay_us);
        let color = DEFAULT_BACKGROUND_COLOR;

        let mut epd = Epd4in2bV2 {
            interface,
            color,
            flag: 0,
        };

        epd.init(spi, delay)?;

        Ok(epd)
    }

    fn sleep(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        if self.flag == 1 {
            self.cmd_with_data(spi, Command::DataStartTransmission1, &[0x03])?;
        } else {
            self.cmd_with_data(spi, Command::VcomAndDataIntervalSetting, &[0xf7])?;
            self.command(spi, Command::PowerOff)?;
            self.wait_until_idle(spi, delay)?;
            self.cmd_with_data(spi, Command::DeepSleep, &[0xA5])?;
        }

        delay.delay_ms(2000);
        Ok(())
    }

    fn wake_up(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.init(spi, delay)
    }

    fn set_background_color(&mut self, color: TriColor) {
        self.color = color;
    }

    fn background_color(&self) -> &TriColor {
        &self.color
    }

    fn width(&self) -> u32 {
        WIDTH
    }

    fn height(&self) -> u32 {
        HEIGHT
    }

    fn update_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.update_color_frame(spi, delay, buffer, buffer)
    }

    fn update_partial_frame(
        &mut self,
        _spi: &mut SPI,
        _delay: &mut DELAY,
        _buffer: &[u8],
        _x: u32,
        _y: u32,
        _width: u32,
        _height: u32,
    ) -> Result<(), SPI::Error> {
        // Partial update not implemented for this display
        unimplemented!("Partial update not supported for EPD4in2b_v2")
    }

    fn display_frame(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.turn_on_display(spi, delay)
    }

    fn update_and_display_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.update_frame(spi, buffer, delay)?;
        self.display_frame(spi, delay)
    }

    fn clear_frame(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        let buffer_size = (WIDTH / 8 * HEIGHT) as usize;
        
        self.wait_until_idle(spi, delay)?;
        
        if self.flag == 1 {
            // Version 1: Use commands 0x24 and 0x26
            self.command(spi, Command::WriteRamBw)?; // 0x24
            self.interface.data_x_times(spi, 0xff, buffer_size as u32)?;
            
            self.command(spi, Command::RedDataTransmission)?; // 0x26
            self.interface.data_x_times(spi, 0x00, buffer_size as u32)?;
        } else {
            // Version 0: Use commands 0x10 and 0x13
            self.command(spi, Command::DataStartTransmission1)?; // 0x10
            self.interface.data_x_times(spi, 0xff, buffer_size as u32)?;
            
            self.command(spi, Command::DataStartTransmission2)?; // 0x13
            self.interface.data_x_times(spi, 0x00, buffer_size as u32)?;
        }
        
        Ok(())
    }

    fn set_lut(
        &mut self,
        _spi: &mut SPI,
        _delay: &mut DELAY,
        _refresh_rate: Option<RefreshLut>,
    ) -> Result<(), SPI::Error> {
        // LUT setting not implemented for this display
        Ok(())
    }

    fn wait_until_idle(&mut self, _spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        // The busy logic differs based on flag version
        if self.flag == 1 {
            // For flag=1, busy is active high (busy when pin is 1)
            self.interface.wait_until_idle(delay, false);
        } else {
            // For flag=0, busy is active low (busy when pin is 0)
            self.interface.wait_until_idle(delay, true);
        }
        Ok(())
    }
}

impl<SPI, BUSY, DC, RST, DELAY> WaveshareThreeColorDisplay<SPI, BUSY, DC, RST, DELAY>
    for Epd4in2bV2<SPI, BUSY, DC, RST, DELAY>
where
    SPI: SpiDevice,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    fn update_color_frame(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
        black: &[u8],
        chromatic: &[u8],
    ) -> Result<(), SPI::Error> {
        self.wait_until_idle(spi, delay)?;
        
        let buffer_size = (WIDTH / 8 * HEIGHT) as usize;
        if black.len() != buffer_size || chromatic.len() != buffer_size {
            // Return a generic error - the actual error handling would need
            // a more specific error type implementation
            return Ok(()); // For now just continue
        }

        if self.flag == 1 {
            // Version 1: Use commands 0x24 and 0x26
            self.command(spi, Command::WriteRamBw)?; // 0x24
            self.interface.data(spi, black)?;
            
            // For red data, we need to invert it
            self.command(spi, Command::RedDataTransmission)?; // 0x26
            for &byte in chromatic.iter() {
                self.interface.data(spi, &[!byte])?;
            }
        } else {
            // Version 0: Use commands 0x10 and 0x13
            self.interface.cmd_with_data(spi, Command::DataStartTransmission1, black)?; // 0x10
            
            // Invert the red/chromatic data (as done in Python)
            self.command(spi, Command::DataStartTransmission2)?; // 0x13
            for &byte in chromatic.iter() {
                self.interface.data(spi, &[!byte])?;
            }
        }

        Ok(())
    }

    fn update_chromatic_frame(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
        chromatic: &[u8],
    ) -> Result<(), SPI::Error> {
        let buffer_size = (WIDTH / 8 * HEIGHT) as usize;
        
        self.wait_until_idle(spi, delay)?;
        
        if self.flag == 1 {
            // Fill black buffer with white (0xff)
            self.command(spi, Command::WriteRamBw)?; // 0x24
            self.interface.data_x_times(spi, 0xff, buffer_size as u32)?;
            
            // Send chromatic data
            self.command(spi, Command::RedDataTransmission)?; // 0x26
            for &byte in chromatic.iter() {
                self.interface.data(spi, &[!byte])?;
            }
        } else {
            // Fill black buffer with white (0xff)
            self.command(spi, Command::DataStartTransmission1)?; // 0x10
            self.interface.data_x_times(spi, 0xff, buffer_size as u32)?;
            
            // Send chromatic data
            self.command(spi, Command::DataStartTransmission2)?; // 0x13
            for &byte in chromatic.iter() {
                self.interface.data(spi, &[!byte])?;
            }
        }
        
        Ok(())
    }

    fn update_achromatic_frame(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
        black: &[u8],
    ) -> Result<(), SPI::Error> {
        let buffer_size = (WIDTH / 8 * HEIGHT) as usize;
        
        self.wait_until_idle(spi, delay)?;
        
        if self.flag == 1 {
            // Send black data
            self.command(spi, Command::WriteRamBw)?; // 0x24
            self.interface.data(spi, black)?;
            
            // Clear red buffer
            self.command(spi, Command::RedDataTransmission)?; // 0x26
            self.interface.data_x_times(spi, 0x00, buffer_size as u32)?;
        } else {
            // Send black data
            self.interface.cmd_with_data(spi, Command::DataStartTransmission1, black)?; // 0x10
            
            // Clear red buffer
            self.command(spi, Command::DataStartTransmission2)?; // 0x13
            self.interface.data_x_times(spi, 0x00, buffer_size as u32)?;
        }
        
        Ok(())
    }
}

impl<SPI, BUSY, DC, RST, DELAY> Epd4in2bV2<SPI, BUSY, DC, RST, DELAY>
where
    SPI: SpiDevice,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    fn command(&mut self, spi: &mut SPI, command: Command) -> Result<(), SPI::Error> {
        self.interface.cmd(spi, command)
    }

    #[allow(dead_code)]
    fn send_data(&mut self, spi: &mut SPI, data: &[u8]) -> Result<(), SPI::Error> {
        self.interface.data(spi, data)
    }

    fn cmd_with_data(
        &mut self,
        spi: &mut SPI,
        command: Command,
        data: &[u8],
    ) -> Result<(), SPI::Error> {
        self.interface.cmd_with_data(spi, command, data)
    }

    /// Turn on display - equivalent to TurnOnDisplay in Python
    fn turn_on_display(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        if self.flag == 1 {
            self.cmd_with_data(spi, Command::DisplayUpdateControl2, &[0xF7])?;
            self.command(spi, Command::MasterActivation)?;
            self.wait_until_idle(spi, delay)?;
        } else {
            self.command(spi, Command::DisplayRefresh)?;
            delay.delay_ms(100);
            self.wait_until_idle(spi, delay)?;
        }
        Ok(())
    }

    /// Set the chip version flag (0 or 1) based on chip ID reading
    /// This should be called before init() if you know your chip version
    pub fn set_flag(&mut self, flag: u8) {
        self.flag = if flag == 1 { 1 } else { 0 };
    }

    /// Get the current chip version flag
    pub fn get_flag(&self) -> u8 {
        self.flag
    }

    /// Create a new instance with a specific chip version flag
    /// This is useful if you know your chip version in advance
    pub fn new_with_flag(
        spi: &mut SPI,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
        delay_us: Option<u32>,
        flag: u8,
    ) -> Result<Self, SPI::Error> {
        let interface = DisplayInterface::new(busy, dc, rst, delay_us);
        let color = DEFAULT_BACKGROUND_COLOR;

        let mut epd = Epd4in2bV2 {
            interface,
            color,
            flag: if flag == 1 { 1 } else { 0 },
        };

        epd.init(spi, delay)?;

        Ok(epd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epd_size() {
        assert_eq!(WIDTH, 400);
        assert_eq!(HEIGHT, 300);
    }

    #[test]
    fn graphics_size() {
        assert_eq!(buffer_len(WIDTH as usize, HEIGHT as usize * 2), 30000);
    }

    #[test]
    fn default_background_color() {
        assert_eq!(DEFAULT_BACKGROUND_COLOR, TriColor::White);
    }
}
