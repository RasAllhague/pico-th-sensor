#![no_std]
#![no_main]

mod display;
mod io_pin;

use core::fmt::Write;
use dht_sensor::{dht22, DhtError, DhtReading};
use display::FmtBuf;
use embedded_graphics::{
    mono_font::{ascii::FONT_9X18_BOLD, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use embedded_hal::digital::v2::OutputPin;
use fugit::RateExtU32;
use hal::Clock;
use io_pin::InOutPin;
use panic_halt as _;
use rp_pico::entry;
use rp_pico::hal;
use rp_pico::hal::pac;
use ssd1306::{prelude::*, Ssd1306};

#[entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins up according to their function on this particular board
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Configure two pins as being I²C, not GPIO
    let sda_pin = pins.gpio16.into_mode::<hal::gpio::FunctionI2C>();
    let scl_pin = pins.gpio17.into_mode::<hal::gpio::FunctionI2C>();

    // Create the I²C driver, using the two pre-configured pins. This will fail
    // at compile time if the pins are in the wrong mode, or if this I²C
    // peripheral isn't available on these pins!
    let i2c = hal::I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        400.kHz(),
        &mut pac.RESETS,
        &clocks.peripheral_clock,
    );

    // Create the I²C display interface:
    let interface = ssd1306::I2CDisplayInterface::new(i2c);

    // Create a driver instance and initialize:
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    // Create a text style for drawing the font:
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_9X18_BOLD)
        .text_color(BinaryColor::On)
        .build();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let mut pin = InOutPin::new(pins.gpio15.into());
    pin.set_high().ok();

    let mut buf = FmtBuf::new();

    delay.delay_ms(1000);

    loop {
        buf.reset();

        let measurement = match dht22::Reading::read(&mut delay, &mut pin) {
            Ok(m) => (0, m.relative_humidity, m.temperature),
            Err(why) => {
                let error = match why {
                    DhtError::PinError(_) => 1,
                    DhtError::ChecksumMismatch => 2,
                    DhtError::Timeout => 3,
                };

                (error, 0.0, 0.0)
            }
        };

        display.clear();

        write!(&mut buf, "Info: {}", measurement.0).unwrap();

        Text::with_baseline(buf.as_str(), Point::new(0, 0), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();

        buf.reset();
        // Format some text into a static buffer:
        write!(&mut buf, "Temp: {}", measurement.2).unwrap();

        Text::with_baseline(buf.as_str(), Point::new(0, 16), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();

        buf.reset();
        write!(&mut buf, "Humid: {}", measurement.1).unwrap();

        Text::with_baseline(buf.as_str(), Point::new(0, 32), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();

        display.flush().unwrap();

        delay.delay_ms(1000);
    }
}
