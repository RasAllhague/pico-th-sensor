#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![no_std]
#![no_main]

mod board;
mod display;
mod errors;
mod rgb_led;

use board::WlanTempSensorBoard;
use core::fmt::{Display, Write};
use display::FmtBuf;
use embedded_graphics::{
    mono_font::{ascii::FONT_9X18_BOLD, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use errors::Error;
use fugit::RateExtU32;
use hal::Clock;
use panic_halt as _;
use rgb_led::RgbLed;
use rp_pico::hal;
use rp_pico::hal::pac;
use rp_pico::{entry, hal::gpio::PinId};
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::{prelude::*, Ssd1306};

pub const DHT_DELAY_MS: u32 = 2000; 

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

    let red_pin = pins.gpio12.into_push_pull_output();
    let green_pin = pins.gpio11.into_push_pull_output();
    let blue_pin = pins.gpio10.into_push_pull_output();

    let rgb_led = RgbLed::new(red_pin, green_pin, blue_pin);

    let wlan_btn_pin = pins.gpio13.into_pull_down_input();
    let power_btn = pins.gpio14.into_pull_down_input();

    delay.delay_ms(DHT_DELAY_MS);

    let buf = FmtBuf::new();

    let board = WlanTempSensorBoard::new(dht22_data_pin, display, rgb_led, wlan_btn_pin, power_btn);

    run(buf, delay, text_style, board);
}

fn run<DHT: PinId, DI, SIZE, RED: PinId, GREEN: PinId, BLUE: PinId, BTN1: PinId, BTN2: PinId>(
    mut buf: FmtBuf,
    mut delay: cortex_m::delay::Delay,
    text_style: MonoTextStyle<BinaryColor>,
    mut board: WlanTempSensorBoard<DHT, DI, SIZE, RED, GREEN, BLUE, BTN1, BTN2>,
) -> !
where
    DI: WriteOnlyDataCommand,
    SIZE: DisplaySize,
{
    loop {
        if let Err(err) = board.display_measurement(&mut buf, &mut delay, text_style) {
            board.blink_error_led(&err, &mut delay);
        } else {
            delay.delay_ms(DHT_DELAY_MS);
        }
    }
}

fn write_formatted_line<T: Display, DI, SIZE>(
    mut buf: &mut FmtBuf,
    text: &str,
    value: T,
    line: u8,
    text_style: MonoTextStyle<BinaryColor>,
    display: &mut Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>,
) -> Result<(), Error>
where
    DI: WriteOnlyDataCommand,
    SIZE: DisplaySize,
{
    write!(&mut buf, "{text}: {value}")?;

    let line = i32::from(line) * 16;

    Text::with_baseline(buf.as_str(), Point::new(0, line), text_style, Baseline::Top)
        .draw(display)?;

    buf.reset();

    Ok(())
}
