mod button;
mod garage_door_controller;
mod mqtt;
mod reed_switch;
mod wifi;

use button::Button;
use esp_idf_svc::{
    hal::{delay::FreeRtos, peripherals::Peripherals},
    sys::EspError,
};
use garage_door_controller::{ControllerError, GarageDoorController};
use mqtt::{GarageCommand, GDMQTT};
use reed_switch::ReedSwitch;
use wifi::WifiHandler;

const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");
const MQTT_ENDPOINT: &str = env!("MQTT_ENDPOINT");
const MQTT_USER: &str = env!("MQTT_USER");
const MQTT_PASS: &str = env!("MQTT_PASS");

const LOOP_DELAY_MS: u32 = 100;
const WIFI_CHECK_INTERVAL: u32 = 50; // ~5 s
const STATUS_PUBLISH_INTERVAL: u32 = 600; // ~10 s

// Helper; log the result of an MQTT publish
fn log_publish_result(name: &str, result: Result<u32, EspError>) {
    match result {
        Ok(_) => log::info!("Published {}", name),
        Err(e) => log::warn!("Failed to publish {}: {:?}", name, e),
    }
}

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
    let mut controller = GarageDoorController::new(open_switch, closed_switch, button)?;

    // WiFi — retry indefinitely so a slow-booting router after a power outage
    // doesn't permanently kill the app (same failure mode as the MQTT timeout).
    log::info!("Connecting to WiFi SSID: {}", WIFI_SSID);
    let mut wifi = WifiHandler::new(peripherals.modem, WIFI_SSID, WIFI_PASSWORD)?;
    loop {
        match wifi.connect() {
            Ok(_) => break,
            Err(e) => log::warn!("WiFi connect failed: {:?}, retrying...", e),
        }
    }
    log::info!("WiFi connected");

    // MQTT
    log::info!("Connecting to MQTT: {}", MQTT_ENDPOINT);
    let mut mqtt = GDMQTT::new("basement_gdopener", MQTT_ENDPOINT, MQTT_USER, MQTT_PASS)?;
    log_publish_result("status", mqtt.publish_status());
    log::info!("MQTT ready");

    // Publish initial door state (state_changed won't fire if door starts Unknown)
    controller.update();
    let initial_state = controller.state();
    log::info!("Initial door state: {:?}", initial_state);
    log_publish_result("state", mqtt.publish_state(initial_state));

    let mut wifi_connected = true;
    let mut mqtt_connected = true;
    let mut loops_since_wifi_check = 0u32;
    let mut loops_since_status = 0u32;

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

        // --- MQTT health: resubscribe after broker reconnect ---
        if mqtt.resubscribe_if_needed() {
            let state = controller.state();
            log_publish_result("state", mqtt.publish_state(state));
            log_publish_result("status", mqtt.publish_status());
            if let Ok(rssi) = wifi.rssi() {
                log_publish_result("rssi", mqtt.publish_rssi(rssi));
            }
        }

        let mqtt_now = mqtt.is_connected();
        if mqtt_connected && !mqtt_now {
            log::warn!("MQTT connection lost");
        } else if !mqtt_connected && mqtt_now {
            log::info!("MQTT connection restored");
        }
        mqtt_connected = mqtt_now;

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
            log_publish_result("state", mqtt.publish_state(state));
            if let Ok(rssi) = wifi.rssi() {
                log_publish_result("rssi", mqtt.publish_rssi(rssi));
            }
            loops_since_status = 0;
        }

        // --- Handle incoming commands ---
        if let Some(cmd) = mqtt.take_command() {
            log::info!("Received command: {:?} (current state: {:?})", cmd, state);

            match cmd {
                GarageCommand::Open => {
                    if let Err(e) = controller.try_open() {
                        match &e {
                            ControllerError::InvalidState(s) => {
                                log::warn!("Open ignored: door is {}", s)
                            }
                            ControllerError::HardwareError(_) => {
                                log::error!("Failed to open garage door: {}", e);
                                log_publish_result(
                                    "error",
                                    mqtt.publish_error(format!("Open cmd error: {}", e)),
                                );
                            }
                        }
                    }
                }
                GarageCommand::Close => {
                    if let Err(e) = controller.try_close() {
                        match &e {
                            ControllerError::InvalidState(s) => {
                                log::warn!("Close ignored: door is {}", s)
                            }
                            ControllerError::HardwareError(_) => {
                                log::error!("Failed to close garage door: {}", e);
                                log_publish_result(
                                    "error",
                                    mqtt.publish_error(format!("Close cmd error: {}", e)),
                                );
                            }
                        }
                    }
                }
            }
        }

        loops_since_wifi_check += 1;
        loops_since_status += 1;
        FreeRtos::delay_ms(LOOP_DELAY_MS);
    }
}
