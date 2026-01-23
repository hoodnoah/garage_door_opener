use std::{sync::{Arc, Mutex}, time::Duration};

use esp_idf_svc::{
    mqtt::client::{EspMqttClient, EventPayload, MqttClientConfiguration, QoS},
    sys::EspError,
};
use lib::state_machine::GDState;

const STATE_TOPIC: &str = "basement_garage/state";
const STATUS_TOPIC: &str = "basement_garage/status";
const ERROR_TOPIC: &str = "basement_garage/error";
const COMMAND_TOPIC: &str = "basement_garage/command";

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

pub struct GDMQTT<'a> {
    client: EspMqttClient<'a>,
    last_command: Arc<Mutex<Option<GarageCommand>>>,
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
            ..Default::default()
        };

        let last_command = Arc::new(Mutex::new(None));
        let last_command_clone = last_command.clone();

        let (mut client, mut connection) = EspMqttClient::new(mqtt_endpoint, &config)?;

        // Subscribe to the command topic
        client.subscribe(COMMAND_TOPIC, QoS::AtLeastOnce)?;

        // Spawn thread to handle incoming messages
        std::thread::spawn(move || {
            while let Ok(event) = connection.next() {
                if let EventPayload::Received {data, ..} = event.payload() {
                    if let Some(cmd) = GarageCommand::from_bytes(data) {
                        *last_command_clone.lock().unwrap() = Some(cmd);
                    }
                }
            }
        });

        Ok(Self { client, last_command })
    }

    pub fn take_command(&mut self) -> Option<GarageCommand>  {
        self.last_command.lock().unwrap().take()
    }

    pub fn publish_state(&mut self, state: GDState) -> Result<u32, EspError> {
        self.client.publish(
            STATE_TOPIC,
            QoS::AtLeastOnce,
            true, // state is set and kept, retain means we keep it
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
}
