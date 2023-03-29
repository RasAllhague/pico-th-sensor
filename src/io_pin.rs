use embedded_hal::digital::v2::{InputPin, OutputPin};
use rp2040_hal::gpio::DynPin;

/// A wrapper for DynPin, implementing both InputPin and OutputPin, to simulate
/// an open-drain pin as needed by the wire protocol the DHT11 sensor speaks.
/// https://how2electronics.com/interfacing-dht11-temperature-humidity-sensor-with-raspberry-pi-pico/
pub struct InOutPin {
    inner: DynPin,
}

impl InOutPin {
    pub fn new(inner: DynPin) -> Self {
        Self { inner }
    }
}

impl InputPin for InOutPin {
    type Error = rp2040_hal::gpio::Error;
    fn is_high(&self) -> Result<bool, <Self as embedded_hal::digital::v2::InputPin>::Error> {
        self.inner.is_high()
    }
    fn is_low(&self) -> Result<bool, <Self as embedded_hal::digital::v2::InputPin>::Error> {
        self.inner.is_low()
    }
}

impl OutputPin for InOutPin {
    type Error = rp2040_hal::gpio::Error;
    fn set_low(&mut self) -> Result<(), <Self as embedded_hal::digital::v2::OutputPin>::Error> {
        // To actively pull the pin low, it must also be configured as a (readable) output pin
        self.inner.into_readable_output();
        // In theory, we should set the pin to low first, to make sure we never actively
        // pull it up. But if we try it on the input pin, we get Err(Gpio(InvalidPinType)).
        self.inner.set_low()?;
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), <Self as embedded_hal::digital::v2::OutputPin>::Error> {
        // To set the open-drain pin to high, just disable the output driver by changing the
        // pin to input mode with pull-up. That way, the DHT11 can still pull the data line down
        // to send its response.
        self.inner.into_pull_up_input();
        Ok(())
    }
}