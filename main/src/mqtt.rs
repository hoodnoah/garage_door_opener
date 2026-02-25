use std::{
    sync::{mpsc, Arc, Mutex},
    time::Duration,
};

use esp_idf_svc::{
    mqtt::client::{EspMqttClient, EventPayload, MqttClientConfiguration, QoS},
    sys::EspError,
};
use lib::state_machine::GDState;

const STATE_TOPIC: &str = "basement_gdopener/state";
const STATUS_TOPIC: &str = "basement_gdopener/status";
const ERROR_TOPIC: &str = "basement_gdopener/error";
const COMMAND_TOPIC: &str = "basement_gdopener/command";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GarageCommand {
    Open,
    Close,
}

impl GarageCommand {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"open" => Some(GarageCommand::Open),
            b"close" => Some(GarageCommand::Close),
            _ => None,
        }
    }
}

pub struct GDMQTT<'a> {
    client: EspMqttClient<'a>,
    command_rx: mpsc::Receiver<GarageCommand>,
}

impl<'a> GDMQTT<'a> {
    pub fn new(
        client_id: &str,
        mqtt_endpoint: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, EspError> {
        let config = MqttClientConfiguration {
            client_id: Some(client_id),
            username: Some(username),
            password: Some(password),
            keep_alive_interval: Some(Duration::from_secs(30)),
            reconnect_timeout: Some(Duration::from_secs(5)),
            ..Default::default()
        };

        let (command_tx, command_rx) = mpsc::channel();
        let connected = Arc::new(Mutex::new(false));
        let connected_flag = connected.clone();

        let (mut client, mut connection) = EspMqttClient::new(mqtt_endpoint, &config)?;

        // The connection thread drives the MQTT state machine; it must be
        // running before we can subscribe or publish.
        std::thread::spawn(move || {
            while let Ok(event) = connection.next() {
                match event.payload() {
                    EventPayload::Connected(_) => {
                        log::info!("MQTT connected");
                        if let Ok(mut flag) = connected_flag.lock() {
                            *flag = true;
                        }
                    }
                    EventPayload::Received { data, .. } => {
                        if let Some(cmd) = GarageCommand::from_bytes(data) {
                            if command_tx.send(cmd).is_err() {
                                break;
                            }
                        }
                    }
                    EventPayload::Error(e) => {
                        log::error!("MQTT error: {:?}", e);
                    }
                    _ => {}
                }
            }
        });

        // Wait for the broker connection before returning.
        let deadline = std::time::Instant::now() + CONNECT_TIMEOUT;
        loop {
            if *connected.lock().unwrap() {
                break;
            }
            if std::time::Instant::now() > deadline {
                log::error!("MQTT connection timed out");
                return Err(EspError::from_infallible::<{ esp_idf_svc::sys::ESP_ERR_TIMEOUT }>());
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        client.subscribe(COMMAND_TOPIC, QoS::AtLeastOnce)?;

        Ok(Self { client, command_rx })
    }

    pub fn take_command(&mut self) -> Option<GarageCommand> {
        self.command_rx.try_recv().ok()
    }

    pub fn publish_state(&mut self, state: GDState) -> Result<u32, EspError> {
        self.client.publish(
            STATE_TOPIC,
            QoS::AtLeastOnce,
            true,
            state.to_string().as_bytes(),
        )
    }

    pub fn publish_status(&mut self) -> Result<u32, EspError> {
        self.client
            .publish(STATUS_TOPIC, QoS::AtLeastOnce, false, b"online")
    }

    pub fn publish_error(&mut self, error_msg: String) -> Result<u32, EspError> {
        self.client
            .publish(ERROR_TOPIC, QoS::AtLeastOnce, false, error_msg.as_bytes())
    }
}
