use dht_sensor::{dht22, DhtReading};
use embedded_graphics::{mono_font::MonoTextStyle, pixelcolor::BinaryColor};
use embedded_hal::digital::v2::OutputPin;
use rp_pico::hal::gpio::{Input, Output, Pin, PinId, PullDown, Readable};
use ssd1306::{
    mode::BufferedGraphicsMode, prelude::WriteOnlyDataCommand, size::DisplaySize, Ssd1306,
};

use crate::{display::FmtBuf, errors::Error, rgb_led::RgbLed, write_formatted_line};

pub struct WlanTempSensorBoard<
    DHT: PinId,
    DI: WriteOnlyDataCommand,
    SIZE: DisplaySize,
    RED: PinId,
    GREEN: PinId,
    BLUE: PinId,
    BTN1: PinId,
    BTN2: PinId,
> {
    dht_data_pin: Pin<DHT, Output<Readable>>,
    display: Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>,
    pub rgb_led: RgbLed<RED, GREEN, BLUE>,
    pub wlan_btn: Pin<BTN1, Input<PullDown>>,
    pub power_btn: Pin<BTN2, Input<PullDown>>,
}

impl<
        DHT: PinId,
        DI: WriteOnlyDataCommand,
        SIZE: DisplaySize,
        RED: PinId,
        GREEN: PinId,
        BLUE: PinId,
        BTN1: PinId,
        BTN2: PinId,
    > WlanTempSensorBoard<DHT, DI, SIZE, RED, GREEN, BLUE, BTN1, BTN2>
{
    pub const fn new(
        dht_data_pin: Pin<DHT, Output<Readable>>,
        display: Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>,
        rgb_led: RgbLed<RED, GREEN, BLUE>,
        wlan_btn: Pin<BTN1, Input<PullDown>>,
        power_btn: Pin<BTN2, Input<PullDown>>,
    ) -> Self {
        Self {
            dht_data_pin,
            display,
            rgb_led,
            wlan_btn,
            power_btn,
        }
    }

    pub fn display_measurement(
        &mut self,
        buf: &mut FmtBuf,
        delay: &mut cortex_m::delay::Delay,
        text_style: MonoTextStyle<BinaryColor>,
    ) -> Result<(), Error> {
        buf.reset();
        self.display.clear();

        let reading = dht22::Reading::read(delay, &mut self.dht_data_pin)?;

        write_formatted_line(
            buf,
            "Temp",
            reading.temperature,
            0,
            text_style,
            &mut self.display,
        )?;
        write_formatted_line(
            buf,
            "Humid",
            reading.relative_humidity,
            1,
            text_style,
            &mut self.display,
        )?;

        self.display.flush()?;

        Ok(())
    }

    pub fn blink_error_led(&mut self, error: &Error, delay: &mut cortex_m::delay::Delay) {
        let interval = error.error_interval();

        match error {
            Error::Dht(_) => {
                self.rgb_led.red.set_high().unwrap();
                delay.delay_ms(interval);
                self.rgb_led.red.set_low().unwrap();
                delay.delay_ms(interval);
            }
            Error::Fmt(_) => {
                self.rgb_led.green.set_high().unwrap();
                delay.delay_ms(interval);
                self.rgb_led.green.set_low().unwrap();
                delay.delay_ms(interval);
            }
            Error::Display(_) => {
                self.rgb_led.blue.set_high().unwrap();
                delay.delay_ms(interval);
                self.rgb_led.blue.set_low().unwrap();
                delay.delay_ms(interval);
            }
        }
    }
}
