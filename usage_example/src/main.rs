use epd_waveshare::epd4in2b_v2::*;
use epd_waveshare::prelude::*;
use linux_embedded_hal::{SpidevDevice, SysfsPin, Delay};
use sysfs_gpio::Direction;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing E-Ink Display EPD4in2B v2...");

    // Initialize SPI
    let mut spi = SpidevDevice::open("/dev/spidev0.0")?;

    // Initialize GPIO pins
    let cs_pin = SysfsPin::new(8);
    cs_pin.with_exported(|| {
        cs_pin.set_direction(Direction::Out)?;
        Ok(())
    })?;

    let busy_pin = SysfsPin::new(24);
    busy_pin.with_exported(|| {
        busy_pin.set_direction(Direction::In)?;
        Ok(())
    })?;

    let dc_pin = SysfsPin::new(25);
    dc_pin.with_exported(|| {
        dc_pin.set_direction(Direction::Out)?;
        Ok(())
    })?;

    let rst_pin = SysfsPin::new(17);
    rst_pin.with_exported(|| {
        rst_pin.set_direction(Direction::Out)?;
        Ok(())
    })?;

    // Initialize delay
    let mut delay = Delay;

    // Create display instance (this already initializes the display)
    let mut display = Epd4in2bV2::new(&mut spi, busy_pin, dc_pin, rst_pin, &mut delay, Some(0))?; // flag = 0 for chip version 1

    println!("Clearing display...");
    display.clear_frame(&mut spi, &mut delay)?;

    // Create buffers for black and red data
    let black_buffer = [0xFF; WIDTH as usize / 8 * HEIGHT as usize]; // White background
    let red_buffer = [0x00; WIDTH as usize / 8 * HEIGHT as usize];   // No red initially

    // You can now draw to the buffers using embedded-graphics or direct pixel manipulation
    // For example, setting some pixels to black:
    // black_buffer[100] = 0x00; // This would make 8 pixels black

    println!("Updating frame data...");
    display.update_color_frame(&mut spi, &mut delay, &black_buffer, &red_buffer)?;

    println!("Displaying frame...");
    display.display_frame(&mut spi, &mut delay)?;

    println!("Display updated successfully!");
    
    Ok(())
}
