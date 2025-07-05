# Using epd-waveshare as a Local Dependency

This guide shows you how to use the epd-waveshare crate as a local dependency in your Rust projects for controlling Waveshare E-Ink displays on Linux (especially Raspberry Pi).

## Directory Structure

```
your-project/
├── Cargo.toml
├── src/
│   └── main.rs
└── epd-waveshare/          # Clone or copy the epd-waveshare repository here
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs
    │   └── epd4in2b_v2/
    └── examples/
```

## Option 1: Local Path Dependency (Recommended)

### 1. Add to your `Cargo.toml`:

```toml
[package]
name = "your-epd-project"
version = "0.1.0"
edition = "2021"

[dependencies]
# Point to the epd-waveshare directory
epd-waveshare = { path = "./epd-waveshare" }

# Required for Linux GPIO/SPI
linux-embedded-hal = "0.4"
sysfs_gpio = "0.6"

# Optional: for drawing graphics
embedded-graphics = "0.8"
```

### 2. Basic Usage Example:

```rust
use epd_waveshare::epd4in2b_v2::*;
use epd_waveshare::prelude::*;
use linux_embedded_hal::{SpidevDevice, SysfsPin, Delay};
use sysfs_gpio::Direction;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize SPI
    let mut spi = SpidevDevice::open("/dev/spidev0.0")?;

    // Initialize GPIO pins (adjust pin numbers for your wiring)
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

    let mut delay = Delay;

    // Create display (automatically initializes)
    // flag: Some(0) for chip version 1, Some(1) for chip version 2
    let mut display = Epd4in2bV2::new(
        &mut spi, 
        busy_pin, 
        dc_pin, 
        rst_pin, 
        &mut delay, 
        Some(0)  // Set to Some(1) for newer chip versions
    )?;

    // Clear the display
    display.clear_frame(&mut spi, &mut delay)?;

    // Create image buffers
    let black_buffer = [0xFF; WIDTH as usize / 8 * HEIGHT as usize]; // All white
    let red_buffer = [0x00; WIDTH as usize / 8 * HEIGHT as usize];   // No red

    // Update and display
    display.update_color_frame(&mut spi, &mut delay, &black_buffer, &red_buffer)?;
    display.display_frame(&mut spi, &mut delay)?;

    Ok(())
}
```

## Option 2: Using Cross for Development

### Building for Raspberry Pi from another platform:

```bash
# Install cross if you haven't already
cargo install cross

# Build for ARM64 (Raspberry Pi 4/5 with 64-bit OS)
cross build --target aarch64-unknown-linux-gnu --release

# Build for ARM32 (older Raspberry Pi or 32-bit OS)
cross build --target armv7-unknown-linux-gnueabihf --release

# Check without building
cross check --target aarch64-unknown-linux-gnu
```

## Hardware Setup

### SPI Configuration
Enable SPI on your Raspberry Pi:
```bash
sudo raspi-config
# Navigate to: Advanced Options > SPI > Enable
```

### Pin Connections (example for 4.2" B v2)
```
EPD Pin    →  Raspberry Pi Pin  →  GPIO
VCC        →  3.3V             →  -
GND        →  Ground           →  -
DIN        →  Pin 19           →  GPIO10 (SPI0_MOSI)
CLK        →  Pin 23           →  GPIO11 (SPI0_SCLK)
CS         →  Pin 24           →  GPIO8  (SPI0_CE0_N)
DC         →  Pin 18           →  GPIO25
RST        →  Pin 11           →  GPIO17
BUSY       →  Pin 18           →  GPIO24
```

Update the GPIO pin numbers in your code to match your wiring.

## Important Configuration

### Chip Version Detection
The EPD4in2B v2 display has two chip versions. You need to set the flag correctly:

```rust
// For chip version 1 (older)
let mut display = Epd4in2bV2::new(&mut spi, busy_pin, dc_pin, rst_pin, &mut delay, Some(0))?;

// For chip version 2 (newer)
let mut display = Epd4in2bV2::new(&mut spi, busy_pin, dc_pin, rst_pin, &mut delay, Some(1))?;

// Auto-detection (uses default - typically version 1)
let mut display = Epd4in2bV2::new(&mut spi, busy_pin, dc_pin, rst_pin, &mut delay, None)?;
```

### Display Dimensions
```rust
// EPD4in2B v2 dimensions
const WIDTH: u32 = 400;
const HEIGHT: u32 = 300;

// Buffer size calculation
let buffer_size = (WIDTH / 8 * HEIGHT) as usize; // 15000 bytes
```

## Drawing Graphics

### Using embedded-graphics

```rust
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
    primitives::{Line, PrimitiveStyle},
};

// Create a framebuffer
let mut black_buffer = [0xFF; 15000]; // White background
let mut red_buffer = [0x00; 15000];   // No red content

// Convert to embedded-graphics format
use embedded_graphics::framebuffer::Framebuffer;
let mut black_fb = Framebuffer::<BinaryColor, _, _, _, 50, 300>::new();
let mut red_fb = Framebuffer::<BinaryColor, _, _, _, 50, 300>::new();

// Draw text
Text::new("Hello, EPD!", Point::new(10, 20), MonoTextStyle::new(&FONT_10X20, BinaryColor::On))
    .draw(&mut black_fb)?;

// Draw a line
Line::new(Point::new(0, 0), Point::new(100, 100))
    .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
    .draw(&mut black_fb)?;

// Convert framebuffer to display buffer
let black_data = black_fb.data();
// Copy to your display buffer...
```

## Common Operations

### Full Display Update
```rust
// Clear, update, and display in one operation
display.clear_frame(&mut spi, &mut delay)?;
display.update_color_frame(&mut spi, &mut delay, &black_buffer, &red_buffer)?;
display.display_frame(&mut spi, &mut delay)?;
```

### Faster Update (skip clear)
```rust
// Just update and display
display.update_color_frame(&mut spi, &mut delay, &black_buffer, &red_buffer)?;
display.display_frame(&mut spi, &mut delay)?;
```

## Running on Raspberry Pi

1. Transfer your compiled binary:
```bash
scp target/aarch64-unknown-linux-gnu/release/your-project pi@raspberrypi.local:~/
```

2. Run with appropriate permissions:
```bash
sudo ./your-project
```

## Troubleshooting

### Permission Issues
Make sure your user is in the `spi` and `gpio` groups:
```bash
sudo usermod -a -G spi,gpio pi
```

### SPI Not Working
Verify SPI is enabled:
```bash
ls /dev/spi*
# Should show: /dev/spidev0.0  /dev/spidev0.1
```

### Display Not Responding
- Check all wiring connections
- Try the other chip version flag (0 or 1)
- Verify power supply (3.3V, sufficient current)
- Check if busy pin is correctly connected and configured

### Build Errors
- Make sure you're using `cross` for cross-compilation
- Verify all dependencies are correctly specified
- Check that the epd-waveshare path is correct in Cargo.toml

## Examples

See the `/examples` directory in the epd-waveshare repository for more detailed examples, including:
- `epd4in2b_v2.rs` - Complete working example
- Graphics drawing examples
- Text rendering examples

This implementation supports both chip versions and has been tested with cross-compilation from macOS to ARM64 Linux targets.
