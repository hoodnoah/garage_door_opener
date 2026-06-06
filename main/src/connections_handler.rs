use esp_idf_svc::{hal::modem::Modem, sys::EspError, systime::EspSystemTime};
use lib::state_machine::GDState;
use std::time::Duration;

use crate::{
    mqtt::{GarageCommand, GDMQTT},
    wifi::WifiHandler,
};

#[derive(Debug)]
pub enum ConnectionError {
    Wifi(EspError),
    Mqtt(EspError),
    WifiRequired,
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionError::Wifi(e) => write!(f, "WiFi error: {}", e),
            ConnectionError::Mqtt(e) => write!(f, "MQTT error: {}", e),
            ConnectionError::WifiRequired => write!(f, "MQTT requires WiFi, which is down"),
        }
    }
}

impl std::error::Error for ConnectionError {}

pub struct WifiVars {
    ssid: String,
    password: String,
    check_interval: Duration,
}

impl WifiVars {
    pub fn new(ssid: String, password: String, check_interval: Duration) -> Self {
        Self {
            ssid,
            password,
            check_interval,
        }
    }
}

pub struct MqttVars {
    endpoint: String,
    user: String,
    password: String,
    status_publish_interval: Duration,
}

impl MqttVars {
    pub fn new(
        endpoint: String,
        user: String,
        password: String,
        status_publish_interval: Duration,
    ) -> Self {
        Self {
            endpoint,
            user,
            password,
            status_publish_interval,
        }
    }
}

pub struct ConnectionsHandler<'a> {
    wifi_vars: WifiVars,
    mqtt_vars: MqttVars,
    wifi_handler: Option<WifiHandler<'a>>,
    mqtt_handler: Option<GDMQTT<'a>>,
    last_wifi_check: Duration,
    last_status_publish: Duration,
}

impl<'a> ConnectionsHandler<'a> {
    pub fn new(wifi_vars: WifiVars, mqtt_vars: MqttVars) -> ConnectionsHandler<'a> {
        ConnectionsHandler {
            wifi_vars: wifi_vars,
            mqtt_vars: mqtt_vars,
            wifi_handler: None,
            mqtt_handler: None,
            last_wifi_check: EspSystemTime {}.now(),
            last_status_publish: EspSystemTime {}.now(),
        }
    }

    pub fn wifi_connect(&mut self, modem: Modem) -> Result<(), ConnectionError> {
        let mut wifi = WifiHandler::new(modem, &self.wifi_vars.ssid, &self.wifi_vars.password)
            .map_err(ConnectionError::Wifi)?;
        wifi.connect().map_err(ConnectionError::Wifi)?;
        self.wifi_handler = Some(wifi);
        Ok(())
    }

    pub fn wifi_is_connected(&self) -> bool {
        match &self.wifi_handler {
            Some(w) => w.is_connected().unwrap_or(false),
            None => false,
        }
    }

    pub fn mqtt_is_connected(&self) -> bool {
        match &self.mqtt_handler {
            Some(m) => m.is_connected(),
            None => false,
        }
    }

    fn tick_wifi(&mut self) -> Result<(), ConnectionError> {
        let now = EspSystemTime {}.now();
        if now - self.last_wifi_check < self.wifi_vars.check_interval {
            return Ok(()); // nothing to do yet
        }
        self.last_wifi_check = now;

        let wifi = self
            .wifi_handler
            .as_mut()
            .ok_or(ConnectionError::WifiRequired)?;
        wifi.ensure_connected().map_err(ConnectionError::Wifi)
    }

    fn tick_mqtt(&mut self, door_state: GDState, state_changed: bool) -> Option<GarageCommand> {
        let rssi = self.wifi_handler.as_ref().and_then(|w| w.rssi().ok());
        let now = EspSystemTime {}.now();
        let mqtt = self.mqtt_handler.as_mut()?;

        // 1. Reconect: resubscribe + repopulate retained topics
        if mqtt.resubscribe_if_needed() {
            let _ = mqtt.publish_state(door_state);
            let _ = mqtt.publish_status();
            if let Some(r) = rssi {
                let _ = mqtt.publish_rssi(r);
            }
        }

        // 2. Event-driven: door moved. Guarded, like each publish.
        if state_changed && mqtt.is_connected() {
            let _ = mqtt.publish_state(door_state);
        }

        // 3. Heartbeat on interval
        if now - self.last_status_publish >= self.mqtt_vars.status_publish_interval {
            self.last_status_publish = now;
            if mqtt.is_connected() {
                let _ = mqtt.publish_status();
                let _ = mqtt.publish_state(door_state);
                if let Some(r) = rssi {
                    let _ = mqtt.publish_rssi(r);
                }
            }
        }

        // 4. Surface command for main to dispatch.
        mqtt.take_command()
    }

    pub fn tick(&mut self, door_state: GDState, state_changed: bool) -> Option<GarageCommand> {
        let _ = self.tick_wifi();
        if !self.wifi_is_connected() {
            return None;
        }
        self.tick_mqtt(door_state, state_changed)
    }

    pub fn mqtt_connect(&mut self) -> Result<(), ConnectionError> {
        if !self.wifi_is_connected() {
            return Err(ConnectionError::WifiRequired);
        }

        let mq = GDMQTT::new(
            "basement_gdopener",
            &self.mqtt_vars.endpoint,
            &self.mqtt_vars.user,
            &self.mqtt_vars.password,
        )
        .map_err(ConnectionError::Mqtt)?;
        self.mqtt_handler = Some(mq);
        Ok(())
    }

    pub fn is_healthy(&self) -> bool {
        self.mqtt_is_connected()
    }
}
