use rp_pico::hal::gpio::{Output, Pin, PinId, PushPull};

pub struct RgbLed<RED: PinId, GREEN: PinId, BLUE: PinId> {
    pub red: Pin<RED, Output<PushPull>>,
    pub green: Pin<GREEN, Output<PushPull>>,
    pub blue: Pin<BLUE, Output<PushPull>>,
}

impl<RED: PinId, GREEN: PinId, BLUE: PinId> RgbLed<RED, GREEN, BLUE> {
    pub fn new(
        red: Pin<RED, Output<PushPull>>,
        green: Pin<GREEN, Output<PushPull>>,
        blue: Pin<BLUE, Output<PushPull>>,
    ) -> RgbLed<RED, GREEN, BLUE> {
        RgbLed { red, green, blue, }
    }
}
