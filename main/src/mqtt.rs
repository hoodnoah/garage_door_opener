use std::time::Duration;

use esp_idf_svc::{
    mqtt::client::{EspMqttClient, MqttClientConfiguration, QoS},
    sys::EspError,
};
use lib::state_machine::GDState;

const STATE_TOPIC: &str = "basement_garage/state";
const STATUS_TOPIC: &str = "basement_garage/status";
const ERROR_TOPIC: &str = "basement_garage/error";

pub struct GDMQTT<'a> {
    client: EspMqttClient<'a>,
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

        let (client, _) = EspMqttClient::new(mqtt_endpoint, &config)?;

        Ok(Self { client })
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
