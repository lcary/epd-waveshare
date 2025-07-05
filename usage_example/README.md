# EPD4in2B v2 Usage Example

This example demonstrates how to use the epd-waveshare crate as a local dependency in your Rust project.

## Setup

1. Make sure you have the epd-waveshare crate available at the parent directory
2. Ensure your target device (Raspberry Pi) has SPI enabled
3. Connect your EPD4in2B v2 display according to the wiring diagram

## Building for Raspberry Pi

From this directory, run:

```bash
# For ARM64 (Raspberry Pi 4/5 with 64-bit OS)
cross build --target aarch64-unknown-linux-gnu --release

# For ARM (older Raspberry Pi or 32-bit OS)
cross build --target armv7-unknown-linux-gnueabihf --release
```

## Running on Raspberry Pi

Transfer the compiled binary to your Raspberry Pi and run with appropriate permissions:

```bash
sudo ./target/aarch64-unknown-linux-gnu/release/epd-example-project
```

## Key Features Used

- **Local dependency**: Uses `epd-waveshare = { path = ".." }` in Cargo.toml
- **Hardware abstraction**: Uses linux-embedded-hal for GPIO and SPI
- **Display control**: Demonstrates initialization, clearing, and displaying
- **Chip version support**: Shows how to set the flag parameter (0 or 1)

## Customization

- Change the `flag` parameter (0 or 1) based on your display chip version
- Modify GPIO pin numbers to match your wiring
- Add embedded-graphics drawing code to create custom content
