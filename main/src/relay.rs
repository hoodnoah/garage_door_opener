use std::time::Duration;

use esp_idf_svc::{
    hal::gpio::{Output, OutputPin, PinDriver},
    sys::EspError,
};

pub struct Relay<'a, T: OutputPin> {
    pin: PinDriver<'a, T, Output>,
    active_high: bool, // true if the relay activates on HIGH, false for LOW
}

impl<'a, T: OutputPin> Relay<'a, T> {
    /// Create a new relay control
    /// active_high: true if relay closes on HIGH signal, false if it closes on LOW
    pub fn new(pin: T, active_high: bool) -> Result<Self, EspError> {
        let mut pin_driver = PinDriver::output(pin)?;

        // start in inactive state
        if active_high {
            pin_driver.set_low()?;
        } else {
            pin_driver.set_high()?;
        }

        Ok(Self {
            pin: pin_driver,
            active_high,
        })
    }

    /// Activate the relay (close the contacts)
    pub fn activate(&mut self) -> Result<(), EspError> {
        if self.active_high {
            self.pin.set_high()
        } else {
            self.pin.set_low()
        }
    }

    /// Deactivate the relay (open the contacts)
    pub fn deactivate(&mut self) -> Result<(), EspError> {
        if self.active_high {
            self.pin.set_low()
        } else {
            self.pin.set_high()
        }
    }

    /// Check if the relay is currently active
    pub fn is_active(&self) -> bool {
        if self.active_high {
            self.pin.is_set_high()
        } else {
            self.pin.is_set_low()
        }
    }

    /// Pulse the relay for a given duration (blocking)
    /// Used to trigger the opener
    pub fn pulse_blocking(&mut self, duration: Duration) -> Result<(), EspError> {
        self.activate()?;
        std::thread::sleep(duration);
        self.deactivate()?;
        Ok(())
    }
}
