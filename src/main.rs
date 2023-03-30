#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![no_std]
#![no_main]

mod display;

use core::fmt::{Display, Write};
use dht_sensor::{dht22, DhtError, DhtReading};
use display::FmtBuf;
use embedded_graphics::{
    mono_font::{ascii::FONT_9X18_BOLD, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use fugit::RateExtU32;
use hal::Clock;
use panic_halt as _;
use rp_pico::entry;
use rp_pico::hal;
use rp_pico::hal::gpio::bank0::Gpio15;
use rp_pico::hal::gpio::{Output, Pin, Readable};
use rp_pico::hal::pac;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::{prelude::*, Ssd1306};

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

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

    let sio = hal::Sio::new(pac.SIO);

    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let sda_pin = pins.gpio16.into_mode::<hal::gpio::FunctionI2C>();
    let scl_pin = pins.gpio17.into_mode::<hal::gpio::FunctionI2C>();

    let i2c = hal::I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        400.kHz(),
        &mut pac.RESETS,
        &clocks.peripheral_clock,
    );

    let interface = ssd1306::I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_9X18_BOLD)
        .text_color(BinaryColor::On)
        .build();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let dht22_data_pin = pins
        .gpio15
        .into_readable_output_in_state(hal::gpio::PinState::High);

    let buf = FmtBuf::new();

    delay.delay_ms(1000);

    run(buf, display, delay, text_style, dht22_data_pin);
}

fn run<DI, SIZE>(
    mut buf: FmtBuf,
    mut display: Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>,
    mut delay: cortex_m::delay::Delay,
    text_style: MonoTextStyle<BinaryColor>,
    mut dht22_data_pin: Pin<Gpio15, Output<Readable>>,
) -> !
where
    DI: WriteOnlyDataCommand,
    SIZE: DisplaySize,
{
    loop {
        buf.reset();
        display.clear();

        match dht22::Reading::read(&mut delay, &mut dht22_data_pin) {
            Ok(reading) => {
                write_formatted_line(
                    &mut buf,
                    "Temp",
                    reading.temperature,
                    0,
                    text_style,
                    &mut display,
                );
                write_formatted_line(
                    &mut buf,
                    "Humid",
                    reading.relative_humidity,
                    1,
                    text_style,
                    &mut display,
                );
            }
            Err(why) => {
                let error = match why {
                    DhtError::PinError(_) => 1,
                    DhtError::ChecksumMismatch => 2,
                    DhtError::Timeout => 3,
                };

                write_formatted_line(&mut buf, "Errorcode:", error, 0, text_style, &mut display);
            }
        };

        display.flush().unwrap();
        delay.delay_ms(1000);
    }
}

fn write_formatted_line<T: Display, DI, SIZE>(
    mut buf: &mut FmtBuf,
    text: &str,
    value: T,
    line: u8,
    text_style: MonoTextStyle<BinaryColor>,
    display: &mut Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>,
) where
    DI: WriteOnlyDataCommand,
    SIZE: DisplaySize,
{
    write!(&mut buf, "{text}: {value}").unwrap();

    let line = i32::from(line) * 16;

    Text::with_baseline(buf.as_str(), Point::new(0, line), text_style, Baseline::Top)
        .draw(display)
        .unwrap();

    buf.reset();
}
