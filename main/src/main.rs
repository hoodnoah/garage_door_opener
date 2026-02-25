mod button;
mod garage_door_controller;
mod mqtt;
mod reed_switch;
mod wifi;

use std::time::Duration;

use button::Button;
use esp_idf_svc::{
    hal::{delay::FreeRtos, peripherals::Peripherals},
    sys::EspError,
};
use garage_door_controller::GarageDoorController;
use lib::state_machine::GDState;
use mqtt::{GarageCommand, GDMQTT};
use reed_switch::ReedSwitch;
use wifi::WifiHandler;

const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");
const MQTT_ENDPOINT: &str = env!("MQTT_ENDPOINT");
const MQTT_USER: &str = env!("MQTT_USER");
const MQTT_PASS: &str = env!("MQTT_PASS");

// GPIO pin assignments
const PIN_CLOSED_SWITCH: u32 = 0; // GPIO0  - "closed" reed switch
const PIN_OPEN_SWITCH: u32 = 21; //  GPIO21 - "open" reed switch
const PIN_BUTTON: u32 = 6; //        GPIO6  - garage door button

const LOOP_DELAY_MS: u32 = 100;
const WIFI_CHECK_INTERVAL: u32 = 50; // ~5 s
const STATUS_PUBLISH_INTERVAL: u32 = 100; // ~10 s

const BUTTON_PULSE_DURATION: Duration = Duration::from_millis(500);

fn log_publish_result(name: &str, result: Result<u32, EspError>) {
    match result {
        Ok(_) => log::info!("Published {}", name),
        Err(e) => log::warn!("Failed to publish {}: {:?}", name, e),
    }
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!(
        "Garage door opener starting (closed=GPIO{}, open=GPIO{}, button=GPIO{})",
        PIN_CLOSED_SWITCH,
        PIN_OPEN_SWITCH,
        PIN_BUTTON,
    );

    let peripherals = Peripherals::take().unwrap();

    // Reed switches: GPIO0 = closed position, GPIO21 = open position
    let closed_switch = ReedSwitch::new(peripherals.pins.gpio0)?;
    let open_switch = ReedSwitch::new(peripherals.pins.gpio21)?;

    // Button: GPIO6 - pulses high to trigger the garage door opener
    let mut button = Button::new(peripherals.pins.gpio6)?;

    // Garage door controller wraps both reed switches and the state machine
    let mut controller = GarageDoorController::new(open_switch, closed_switch)?;

    // WiFi
    log::info!("Connecting to WiFi SSID: {}", WIFI_SSID);
    let mut wifi = WifiHandler::new(peripherals.modem, WIFI_SSID, WIFI_PASSWORD)?;
    wifi.connect()?;
    log::info!("WiFi connected");

    // MQTT
    log::info!("Connecting to MQTT: {}", MQTT_ENDPOINT);
    let mut mqtt = GDMQTT::new("basement_gdopener", MQTT_ENDPOINT, MQTT_USER, MQTT_PASS)?;
    log_publish_result("status", mqtt.publish_status());
    log::info!("MQTT ready");

    let mut wifi_connected = true;
    let mut loops_since_wifi_check = 0u32;
    let mut loops_since_status = 0u32;

    log::info!("Entering main loop");

    loop {
        // --- Periodic WiFi health check ---
        if loops_since_wifi_check >= WIFI_CHECK_INTERVAL {
            match wifi.ensure_connected() {
                Ok(_) => {
                    if !wifi_connected {
                        log::info!("WiFi connection restored");
                    }
                    wifi_connected = true;
                }
                Err(e) => {
                    log::error!("WiFi check failed: {:?}", e);
                    wifi_connected = false;
                }
            }
            loops_since_wifi_check = 0;
        }

        // --- Update door state machine ---
        let state_changed = controller.update();
        let state = controller.state();

        if state_changed {
            log::info!("Door state -> {:?}", state);
            log_publish_result("state", mqtt.publish_state(state));
        }

        // --- Periodic status heartbeat ---
        if loops_since_status >= STATUS_PUBLISH_INTERVAL {
            log_publish_result("status", mqtt.publish_status());
            loops_since_status = 0;
        }

        // --- Handle incoming commands ---
        if let Some(cmd) = mqtt.take_command() {
            log::info!("Received command: {:?} (current state: {:?})", cmd, state);

            // Only pulse if the command makes sense for the current state.
            // Both Open and Close result in a single button press (toggle).
            let should_pulse = match cmd {
                GarageCommand::Open => {
                    !matches!(state, GDState::Open | GDState::Opening | GDState::Unknown)
                }
                GarageCommand::Close => {
                    !matches!(state, GDState::Closed | GDState::Closing | GDState::Unknown)
                }
            };

            if should_pulse {
                log::info!(
                    "Pulsing garage door button for {:?}ms",
                    BUTTON_PULSE_DURATION
                );
                if let Err(e) = button.pulse_blocking(BUTTON_PULSE_DURATION) {
                    log::error!("Button pulse failed: {:?}", e);
                    log_publish_result(
                        "error",
                        mqtt.publish_error(format!("Button error: {:?}", e)),
                    );
                }
            } else {
                log::warn!("Command {:?} ignored in state {:?}", cmd, state);
                log_publish_result(
                    "error",
                    mqtt.publish_error(format!(
                        "Command {:?} rejected: door state {:?}",
                        cmd, state
                    )),
                );
            }
        }

        loops_since_wifi_check += 1;
        loops_since_status += 1;
        FreeRtos::delay_ms(LOOP_DELAY_MS);
    }
}
