mod button;
mod connections_handler;
mod garage_door_controller;
mod mqtt;
mod reed_switch;
mod wifi;

use std::time::Duration;

use button::Button;
use connections_handler::{MqttVars, WifiVars};
use esp_idf_svc::{
    hal::{delay::FreeRtos, gpio::IOPin, peripherals::Peripherals},
    systime::EspSystemTime,
};
use garage_door_controller::{ControllerError, GarageDoorController};
use mqtt::GarageCommand;
use reed_switch::ReedSwitch;

use crate::connections_handler::ConnectionsHandler;

const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");
const MQTT_ENDPOINT: &str = env!("MQTT_ENDPOINT");
const MQTT_USER: &str = env!("MQTT_USER");
const MQTT_PASS: &str = env!("MQTT_PASS");

const LOOP_DELAY_MS: u32 = 100;
const WIFI_CHECK_INTERVAL: Duration = Duration::from_secs(5);
const WIFI_RETRY_INTERVAL_MS: u32 = 10_000;
const STATUS_PUBLISH_INTERVAL: Duration = Duration::from_secs(60);
const REBOOT_TIMEOUT: Duration = Duration::from_secs(5 * 60);

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    // Reed switches: GPIO3 = closed position, GPIO1 = open position
    let closed_switch = ReedSwitch::new(peripherals.pins.gpio3.downgrade())?;
    let open_switch = ReedSwitch::new(peripherals.pins.gpio1.downgrade())?;

    // Button: GPIO20 - pulses high to trigger the garage door opener
    let button = Button::new(peripherals.pins.gpio20.downgrade())?;

    // Garage door controller wraps both reed switches and the state machine
    let mut door_controller = GarageDoorController::new(open_switch, closed_switch, button)?;

    // Connections controller
    let wifi_vars = WifiVars::new(
        WIFI_SSID.to_string(),
        WIFI_PASSWORD.to_string(),
        WIFI_CHECK_INTERVAL,
    );
    let mqtt_vars = MqttVars::new(
        MQTT_ENDPOINT.to_string(),
        MQTT_USER.to_string(),
        MQTT_PASS.to_string(),
        STATUS_PUBLISH_INTERVAL,
    );
    let mut connections_handler = ConnectionsHandler::new(wifi_vars, mqtt_vars);

    // WiFi — retry indefinitely so a slow-booting router after a power outage
    // doesn't kill the whole app permanently
    match connections_handler.wifi_connect(peripherals.modem) {
        Ok(_) => {}
        Err(_) if connections_handler.has_wifi_handler() => loop {
            // retry if wifi_handler new worked
            match connections_handler.wifi_reconnect() {
                Ok(_) => break,
                Err(e) => {
                    log::warn!("WiFi retrying: {:?}", e);
                    FreeRtos::delay_ms(WIFI_RETRY_INTERVAL_MS);
                }
            }
        },
        Err(e) => {
            // new() itself failed, modem is gone. Needs reboot.
            log::error!("WiFi init failed unrecoverably: {:?}; restarting", e);
            esp_idf_svc::hal::reset::restart();
        }
    }

    // MQTT
    log::info!("Connecting to MQTT: {}", MQTT_ENDPOINT);
    loop {
        match connections_handler.mqtt_connect() {
            Ok(_) => break,
            Err(e) => log::warn!("MQTT connect failed: {:?}, retrying...", e),
        }
    }
    log::info!("MQTT ready");

    let mut last_ok = EspSystemTime {}.now();

    loop {
        // --- Update door state machine ---
        let state_changed = door_controller.update();
        let door_state = door_controller.state();

        if let Some(cmd) = connections_handler.tick(door_state, state_changed) {
            log::info!(
                "Received command: {:?} (current state: {:?})",
                cmd,
                door_state
            );

            match cmd {
                GarageCommand::Open => {
                    if let Err(e) = door_controller.try_open() {
                        match &e {
                            ControllerError::InvalidState(s) => {
                                log::warn!("Open ignored: door is {}", s)
                            }
                            ControllerError::HardwareError(_) => {
                                log::error!("Failed to open garage door: {}", e);
                                connections_handler
                                    .mqtt_publish_error(format!("Open cmd error: {}", e));
                            }
                        }
                    }
                }
                GarageCommand::Close => {
                    if let Err(e) = door_controller.try_close() {
                        match &e {
                            ControllerError::InvalidState(s) => {
                                log::warn!("Close ignored: door is {}", s)
                            }
                            ControllerError::HardwareError(_) => {
                                log::error!("Failed to close garage door: {}", e);
                                connections_handler
                                    .mqtt_publish_error(format!("Close cmd error: {}", e));
                            }
                        }
                    }
                }
            }
        }

        if connections_handler.is_healthy() {
            last_ok = EspSystemTime {}.now();
        } else if (EspSystemTime {}.now() - last_ok) >= REBOOT_TIMEOUT {
            log::error!("Connectivity dead too long; restarting");
            esp_idf_svc::hal::reset::restart();
        }

        FreeRtos::delay_ms(LOOP_DELAY_MS);
    }
}
