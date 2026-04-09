use std::time::Duration;

use esp_idf_svc::{
    hal::{
        delay::FreeRtos,
        gpio::{AnyIOPin, Output, PinDriver},
    },
    sys::EspError,
};

pub struct Button<'a> {
    pin: PinDriver<'a, AnyIOPin, Output>,
}

impl<'a> Button<'a> {
    pub fn new(pin: AnyIOPin) -> Result<Self, EspError> {
        let pin_driver = PinDriver::output(pin)?;
        Ok(Self { pin: pin_driver })
    }

    /// Drive the pin high for `duration`, then return it low.
    pub fn pulse_blocking(&mut self, duration: Duration) -> Result<(), EspError> {
        self.pin.set_high()?;
        FreeRtos::delay_ms(duration.as_millis() as u32);
        self.pin.set_low()?;
        Ok(())
    }
}
