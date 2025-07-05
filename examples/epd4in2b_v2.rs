#!/usr/bin/env cargo

//! Example for the Waveshare 4.2" E-Ink Display Module B v2.2
//! 
//! This example demonstrates how to use the epd4in2b_v2 driver to display
//! graphics on the three-color e-ink display.

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyleBuilder, Rectangle},
    text::{Baseline, Text},
};
use embedded_hal::delay::DelayNs;
use epd_waveshare::{epd4in2b_v2::*, prelude::*};
use linux_embedded_hal::{
    spidev::{self, SpidevOptions},
    sysfs_gpio::Direction,
    Delay, SpidevDevice, SysfsPin,
};

// Activate SPI, GPIO in raspi-config
// needs to be run with sudo because of some sysfs_gpio permission problems and follow-up timing problems
// see https://github.com/rust-embedded/rust-sysfs-gpio/issues/5 and follow-up issues

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure SPI
    let mut spi = SpidevDevice::open("/dev/spidev0.0").expect("spidev directory");
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(4_000_000)
        .mode(spidev::SpiModeFlags::SPI_MODE_0)
        .build();
    spi.configure(&options).expect("spi configuration");

    // Configure Digital I/O Pin to be used as Chip Select for SPI
    let cs = SysfsPin::new(8); // BCM8
    cs.export().expect("cs export");
    while !cs.is_exported() {}
    cs.set_direction(Direction::Out)
        .expect("cs Direction");
    cs.set_value(1).expect("cs Value set to 1");

    let busy = SysfsPin::new(24); // BCM24
    busy.export().expect("busy export");
    while !busy.is_exported() {}
    busy.set_direction(Direction::In)
        .expect("busy Direction");

    let dc = SysfsPin::new(25); // BCM25
    dc.export().expect("dc export");
    while !dc.is_exported() {}
    dc.set_direction(Direction::Out)
        .expect("dc Direction");
    dc.set_value(0).expect("dc Value set to 0");

    let rst = SysfsPin::new(17); // BCM17
    rst.export().expect("rst export");
    while !rst.is_exported() {}
    rst.set_direction(Direction::Out)
        .expect("rst Direction");
    rst.set_value(0).expect("rst Value set to 0");

    let mut delay = Delay {};

    // Setup EPD
    let mut epd = Epd4in2bV2::new(&mut spi, busy, dc, rst, &mut delay, None)?;

    println!("Test all-black and all-white-red");

    // Clear the full screen
    epd.clear_frame(&mut spi, &mut delay)?;
    epd.display_frame(&mut spi, &mut delay)?;

    // Introduce a delay between operations
    delay.delay_ms(1000);

    println!("Now test new graphics with embedded-graphics");

    // Use display graphics from embedded-graphics
    let mut tricolor_display = Display4in2bV2::default();

    // Draw some graphics
    let style_black = PrimitiveStyleBuilder::new()
        .stroke_color(TriColor::Black)
        .stroke_width(1)
        .build();

    let style_red = PrimitiveStyleBuilder::new()
        .stroke_color(TriColor::Chromatic)
        .stroke_width(1)
        .build();

    // Draw a black rectangle
    Rectangle::new(Point::new(50, 50), Size::new(100, 80))
        .into_styled(style_black)
        .draw(&mut tricolor_display)?;

    // Draw a red circle
    Circle::new(Point::new(200, 100), 50)
        .into_styled(style_red)
        .draw(&mut tricolor_display)?;

    // Draw some lines
    Line::new(Point::new(0, 0), Point::new(399, 299))
        .into_styled(style_black)
        .draw(&mut tricolor_display)?;

    Line::new(Point::new(399, 0), Point::new(0, 299))
        .into_styled(style_red)
        .draw(&mut tricolor_display)?;

    // Add some text
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(TriColor::Black)
        .build();

    Text::with_baseline("Hello Rust!", Point::new(20, 20), text_style, Baseline::Top)
        .draw(&mut tricolor_display)?;

    let text_style_red = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(TriColor::Chromatic)
        .build();

    Text::with_baseline("EPD 4in2b v2.2", Point::new(20, 200), text_style_red, Baseline::Top)
        .draw(&mut tricolor_display)?;

    // Display the frame
    epd.update_color_frame(
        &mut spi,
        &mut delay,
        &tricolor_display.bw_buffer(),
        &tricolor_display.chromatic_buffer(),
    )?;
    epd.display_frame(&mut spi, &mut delay)?;

    println!("Finished - going to sleep");
    epd.sleep(&mut spi, &mut delay)?;

    Ok(())
}
