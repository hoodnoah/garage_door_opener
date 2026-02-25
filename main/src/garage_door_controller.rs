use std::time::{Duration, Instant};

use crate::{
    button::Button,
    reed_switch::{ReedSwitch, SwitchState},
};
use esp_idf_svc::{
    hal::gpio::{InputPin, OutputPin},
    sys::EspError,
};
use std::fmt;

#[derive(Debug)]
pub enum ControllerError {
    InvalidState(GDState),
    HardwareError(EspError),
}

impl fmt::Display for ControllerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ControllerError::InvalidState(state) => {
                write!(f, "command not valid in current state: {}", state)
            }
            ControllerError::HardwareError(e) => write!(f, "hardware error: {}", e),
        }
    }
}
use lib::{
    state_machine::{DoorPosition, GDState},
    GDStateMachine,
};

pub struct GarageDoorController<'a, T1, T2, T3>
where
    T1: InputPin + OutputPin,
    T2: InputPin + OutputPin,
    T3: InputPin + OutputPin,
{
    open_switch: ReedSwitch<'a, T1>,
    closed_switch: ReedSwitch<'a, T2>,
    button: Button<'a, T3>,
    state_machine: GDStateMachine,
    last_state_change: Instant,
}

impl<'a, T1, T2, T3> GarageDoorController<'a, T1, T2, T3>
where
    T1: InputPin + OutputPin,
    T2: InputPin + OutputPin,
    T3: InputPin + OutputPin,
{
    const TRANSITION_TIMEOUT_SECONDS: u64 = 20;
    const BUTTON_PULSE_DURATION: Duration = Duration::from_millis(500);

    pub fn new(
        open_switch: ReedSwitch<'a, T1>,
        closed_switch: ReedSwitch<'a, T2>,
        button: Button<'a, T3>,
    ) -> Result<Self, EspError> {
        Ok(Self {
            open_switch,
            closed_switch,
            button,
            state_machine: GDStateMachine::new(),
            last_state_change: Instant::now(),
        })
    }

    fn read_position(&self) -> DoorPosition {
        use SwitchState::*;

        match (self.open_switch.state(), self.closed_switch.state()) {
            (CircuitClosed, CircuitOpen) => DoorPosition::FullyOpen,
            (CircuitOpen, CircuitClosed) => DoorPosition::FullyClosed,
            (CircuitOpen, CircuitOpen) => DoorPosition::Moving,
            (CircuitClosed, CircuitClosed) => DoorPosition::Unknown, // undefined; both switches closed¿
            _ => DoorPosition::Unknown,                              // cannot be inferred
        }
    }

    /// Update switches and state machine, returns true if state changed
    pub fn update(&mut self) -> bool {
        let current_instant = Instant::now();

        // Update hardware sensors
        self.open_switch.update(current_instant);
        self.closed_switch.update(current_instant);

        // Get door position
        let position = self.check_door_position(current_instant);

        let state_changed = self.state_machine.update(position);

        if state_changed {
            self.last_state_change = current_instant;
        }

        state_changed
    }

    /// Check if door has been transitioning for too long
    fn check_door_position(&self, current_instant: Instant) -> DoorPosition {
        let position = self.read_position();
        let elapsed = current_instant - self.last_state_change;
        let timeout_duration = Duration::from_secs(Self::TRANSITION_TIMEOUT_SECONDS);

        // If we've been in a transitioning state too long, report as unknown
        match self.state_machine.state() {
            GDState::Opening | GDState::Closing => {
                if elapsed > timeout_duration {
                    DoorPosition::Unknown
                } else {
                    position
                }
            }
            _ => position,
        }
    }

    pub fn state(&self) -> GDState {
        self.state_machine.state()
    }

    // attempts to open the door, if state allows.
    pub fn try_open(&mut self) -> Result<(), ControllerError> {
        if !self.state_machine.can_open() {
            return Err(ControllerError::InvalidState(self.state_machine.state()));
        }
        self.button
            .pulse_blocking(Self::BUTTON_PULSE_DURATION)
            .map_err(ControllerError::HardwareError)
    }

    // attempts to close the door, if state allows.
    pub fn try_close(&mut self) -> Result<(), ControllerError> {
        if !self.state_machine.can_close() {
            return Err(ControllerError::InvalidState(self.state_machine.state()));
        }
        self.button
            .pulse_blocking(Self::BUTTON_PULSE_DURATION)
            .map_err(ControllerError::HardwareError)
    }
}
