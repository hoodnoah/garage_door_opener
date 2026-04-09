use std::time::{Duration, Instant};

use esp_idf_svc::{
    hal::gpio::{AnyIOPin, Input, PinDriver, Pull},
    sys::EspError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchState {
    CircuitOpen,
    CircuitClosed,
}

pub struct ReedSwitch<'a> {
    pin: PinDriver<'a, AnyIOPin, Input>,
    state: SwitchState,
    last_raw: SwitchState,
    last_edge: Instant,
}

impl<'a> ReedSwitch<'a> {
    const DEBOUNCE_DELAY_US: u64 = 150_000; // 150ms

    pub fn new(pin: AnyIOPin) -> Result<Self, EspError> {
        let mut pin_driver = PinDriver::input(pin)?;
        pin_driver.set_pull(Pull::Up)?;

        let initial = if pin_driver.is_low() {
            SwitchState::CircuitClosed
        } else {
            SwitchState::CircuitOpen
        };

        Ok(Self {
            pin: pin_driver,
            state: initial,
            last_raw: initial,
            last_edge: Instant::now(),
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

        // track when the raw signal last changed
        if new_reading != self.last_raw {
            self.last_raw = new_reading;
            self.last_edge = read_time;
        }

        // only accept if raw has been stable for the full debounce window
        let stable_elapsed = read_time - self.last_edge;
        if self.last_raw != self.state
            && stable_elapsed > Duration::from_micros(Self::DEBOUNCE_DELAY_US)
        {
            self.state = self.last_raw;
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
