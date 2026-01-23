use std::time::{Duration, Instant};

use esp_idf_svc::{
    hal::gpio::{Input, InputPin, OutputPin, PinDriver, Pull},
    sys::EspError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchState {
    CircuitOpen,
    CircuitClosed,
    Uninitialized,
}

pub struct ReedSwitch<'a, T: InputPin + OutputPin> {
    pin: PinDriver<'a, T, Input>,
    state: SwitchState,
    prev_state: Option<SwitchState>,
    last_changed: Instant,
}

impl<'a, T: InputPin + OutputPin> ReedSwitch<'a, T> {
    const DEBOUNCE_DELAY_US: u64 = 50_000;

    pub fn new(pin: T) -> Result<Self, EspError> {
        let mut pin_driver = PinDriver::input(pin)?;
        pin_driver.set_pull(Pull::Up)?;

        Ok(Self {
            pin: pin_driver,
            state: SwitchState::Uninitialized,
            prev_state: None,
            last_changed: Instant::now(),
        })
    }

    /// Read the current switch state from hardware
    fn read_hardware(&self) -> SwitchState {
        if self.pin.is_low() {
            // pulling up; normally high when magnet absent
            SwitchState::CircuitClosed // magnet present
        } else {
            // pulling up; normally low when magnet present
            SwitchState::CircuitOpen // no magnet
        }
    }

    /// Update switch state; returns true if changed
    pub fn update(&mut self, read_time: Instant) -> bool {
        let new_reading = self.read_hardware();
        let elapsed = read_time - self.last_changed;

        if new_reading != self.state && elapsed > Duration::from_micros(Self::DEBOUNCE_DELAY_US) {
            self.prev_state = Some(self.state);
            self.state = new_reading;
            self.last_changed = read_time;
            true
        } else {
            false
        }
    }

    /// Get current switch state
    pub fn state(&self) -> SwitchState {
        self.state
    }
}
