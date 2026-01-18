#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorPosition {
    FullyClosed,
    FullyOpen,
    Moving,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GDState {
    Open,
    Closed,
    Opening,
    Closing,
    SafetyStoppedWhileOpening,
    SafetyStoppedWhileClosing,
    Unknown,
}

pub struct GDStateMachine {
    state: GDState,
}

impl GDStateMachine {
    pub fn new() -> Self {
        GDStateMachine {
            state: GDState::Unknown,
        }
    }

    pub fn update(&mut self, position: DoorPosition) -> bool {
        let og_state = self.state;
        match (self.state, position) {
            (GDState::Unknown, p) => match p {
                DoorPosition::FullyOpen => self.state = GDState::Open,
                DoorPosition::FullyClosed => self.state = GDState::Closed,
                DoorPosition::Moving => self.state = GDState::Unknown,
                DoorPosition::Unknown => (),
            },
            (GDState::Closed, p) => match p {
                DoorPosition::FullyOpen => self.state = GDState::Open,
                DoorPosition::FullyClosed => (),
                DoorPosition::Moving => self.state = GDState::Opening,
                DoorPosition::Unknown => self.state = GDState::Unknown,
            },
            (GDState::Open, p) => match p {
                DoorPosition::FullyOpen => (),
                DoorPosition::FullyClosed => self.state = GDState::Closed,
                DoorPosition::Moving => self.state = GDState::Closing,
                DoorPosition::Unknown => self.state = GDState::Unknown,
            },
            (GDState::Closing, p) => match p {
                DoorPosition::FullyOpen => self.state = GDState::SafetyStoppedWhileClosing,
                DoorPosition::FullyClosed => self.state = GDState::Closed,
                DoorPosition::Moving => (),
                DoorPosition::Unknown => self.state = GDState::SafetyStoppedWhileClosing,
            },
            (GDState::Opening, p) => match p {
                DoorPosition::FullyOpen => self.state = GDState::Open,
                DoorPosition::FullyClosed => self.state = GDState::SafetyStoppedWhileOpening,
                DoorPosition::Moving => (),
                DoorPosition::Unknown => self.state = GDState::SafetyStoppedWhileOpening,
            },
            (GDState::SafetyStoppedWhileClosing, p) => match p {
                DoorPosition::FullyOpen => self.state = GDState::Open,
                DoorPosition::FullyClosed => self.state = GDState::Closed,
                DoorPosition::Moving => self.state = GDState::Opening,
                DoorPosition::Unknown => (),
            },
            (GDState::SafetyStoppedWhileOpening, p) => match p {
                DoorPosition::FullyOpen => self.state = GDState::Open,
                DoorPosition::FullyClosed => self.state = GDState::Closed,
                DoorPosition::Moving => self.state = GDState::Closing,
                DoorPosition::Unknown => (),
            },
        };

        og_state != self.state
    }

    pub fn state(&self) -> GDState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_with_state(state: GDState) -> GDStateMachine {
        let mut sm = GDStateMachine::new();
        sm.state = state;

        sm
    }

    #[test]
    fn starts_unknown() {
        let sm = GDStateMachine::new();

        assert_eq!(sm.state, GDState::Unknown);
    }

    #[test]
    fn unknown_to_closed() {
        let mut sm = GDStateMachine::new();
        sm.update(DoorPosition::FullyClosed);

        assert_eq!(sm.state, GDState::Closed);
    }

    #[test]
    fn unknown_to_open() {
        let mut sm = GDStateMachine::new();
        sm.update(DoorPosition::FullyOpen);

        assert_eq!(sm.state, GDState::Open);
    }

    #[test]
    fn unknown_and_moving_is_unknown() {
        let mut sm = GDStateMachine::new();
        sm.update(DoorPosition::Moving);

        assert_eq!(sm.state, GDState::Unknown);
    }

    #[test]
    fn unknown_and_unknown_is_unknown() {
        let mut sm = GDStateMachine::new();
        sm.update(DoorPosition::Unknown);

        assert_eq!(sm.state, GDState::Unknown);
    }

    #[test]
    fn closed_and_closed_is_closed() {
        let mut sm = new_with_state(GDState::Closed);

        sm.update(DoorPosition::FullyClosed);

        assert_eq!(sm.state, GDState::Closed);
    }

    #[test]
    fn closed_and_moving_is_opening() {
        let mut sm = new_with_state(GDState::Closed);

        sm.update(DoorPosition::Moving);

        assert_eq!(sm.state, GDState::Opening);
    }

    #[test]
    fn closed_and_open_is_open() {
        let mut sm = new_with_state(GDState::Closed);

        sm.update(DoorPosition::FullyOpen);

        assert_eq!(sm.state, GDState::Open);
    }

    #[test]
    fn closed_and_unknown_is_unknown() {
        let mut sm = new_with_state(GDState::Closed);

        sm.update(DoorPosition::Unknown);

        assert_eq!(sm.state, GDState::Unknown);
    }

    #[test]
    fn open_and_open_is_open() {
        let mut sm = new_with_state(GDState::Open);

        sm.update(DoorPosition::FullyOpen);

        assert_eq!(sm.state, GDState::Open);
    }

    #[test]
    fn open_and_moving_is_closing() {
        let mut sm = new_with_state(GDState::Open);

        sm.update(DoorPosition::Moving);

        assert_eq!(sm.state, GDState::Closing);
    }

    #[test]
    fn open_and_closed_is_closed() {
        let mut sm = new_with_state(GDState::Open);

        sm.update(DoorPosition::FullyClosed);

        assert_eq!(sm.state, GDState::Closed);
    }

    #[test]
    fn open_and_unknown_is_unknown() {
        let mut sm = new_with_state(GDState::Open);

        sm.update(DoorPosition::Unknown);

        assert_eq!(sm.state, GDState::Unknown);
    }

    #[test]
    fn closing_and_moving_is_closing() {
        let mut sm = new_with_state(GDState::Closing);

        sm.update(DoorPosition::Moving);

        assert_eq!(sm.state, GDState::Closing);
    }

    #[test]
    fn closing_and_closed_is_closed() {
        let mut sm = new_with_state(GDState::Closing);

        sm.update(DoorPosition::FullyClosed);

        assert_eq!(sm.state, GDState::Closed);
    }

    #[test]
    fn closing_and_unknown_is_safety_stopped_while_closing() {
        let mut sm = new_with_state(GDState::Closing);

        sm.update(DoorPosition::Unknown);

        assert_eq!(sm.state, GDState::SafetyStoppedWhileClosing);
    }

    #[test]
    fn closing_and_open_is_safety_stopped_while_closing() {
        let mut sm = new_with_state(GDState::Closing);

        sm.update(DoorPosition::FullyOpen);

        assert_eq!(sm.state, GDState::SafetyStoppedWhileClosing)
    }

    #[test]
    fn opening_and_open_is_open() {
        let mut sm = new_with_state(GDState::Opening);

        sm.update(DoorPosition::FullyOpen);

        assert_eq!(sm.state, GDState::Open);
    }

    #[test]
    fn opening_and_closed_is_safety_stopped_while_opening() {
        let mut sm = new_with_state(GDState::Opening);

        sm.update(DoorPosition::FullyClosed);

        assert_eq!(sm.state, GDState::SafetyStoppedWhileOpening);
    }

    #[test]
    fn opening_and_moving_is_opening() {
        let mut sm = new_with_state(GDState::Opening);

        sm.update(DoorPosition::Moving);

        assert_eq!(sm.state, GDState::Opening);
    }

    #[test]
    fn opening_and_unknown_is_safety_stopped_while_opening() {
        let mut sm = new_with_state(GDState::Opening);

        sm.update(DoorPosition::Unknown);

        assert_eq!(sm.state, GDState::SafetyStoppedWhileOpening);
    }

    #[test]
    fn stopped_closing_and_closed_is_closed() {
        let mut sm = new_with_state(GDState::SafetyStoppedWhileClosing);

        sm.update(DoorPosition::FullyClosed);

        assert_eq!(sm.state, GDState::Closed);
    }

    #[test]
    fn stopped_closing_and_moving_is_opening() {
        let mut sm = new_with_state(GDState::SafetyStoppedWhileClosing);

        sm.update(DoorPosition::Moving);

        assert_eq!(sm.state, GDState::Opening);
    }

    #[test]
    fn stopped_closing_and_unknown_is_stopped_closing() {
        let mut sm = new_with_state(GDState::SafetyStoppedWhileClosing);

        sm.update(DoorPosition::Unknown);

        assert_eq!(sm.state, GDState::SafetyStoppedWhileClosing);
    }

    #[test]
    fn stopped_closing_and_open_is_open() {
        let mut sm = new_with_state(GDState::SafetyStoppedWhileClosing);

        sm.update(DoorPosition::FullyOpen);

        assert_eq!(sm.state, GDState::Open);
    }

    #[test]
    fn stopped_opening_and_closed_is_closed() {
        let mut sm = new_with_state(GDState::SafetyStoppedWhileOpening);

        sm.update(DoorPosition::FullyClosed);

        assert_eq!(sm.state, GDState::Closed);
    }

    #[test]
    fn stopped_opening_and_open_is_open() {
        let mut sm = new_with_state(GDState::SafetyStoppedWhileOpening);

        sm.update(DoorPosition::FullyOpen);

        assert_eq!(sm.state, GDState::Open);
    }

    #[test]
    fn stopped_opening_and_moving_is_closing() {
        let mut sm = new_with_state(GDState::SafetyStoppedWhileOpening);

        sm.update(DoorPosition::Moving);

        assert_eq!(sm.state, GDState::Closing);
    }

    #[test]
    fn stopped_opening_and_unknown_is_stopped_opening() {
        let mut sm = new_with_state(GDState::SafetyStoppedWhileOpening);

        sm.update(DoorPosition::Unknown);

        assert_eq!(sm.state, GDState::SafetyStoppedWhileOpening);
    }
}
