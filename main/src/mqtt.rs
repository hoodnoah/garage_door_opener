use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
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
const RSSI_TOPIC: &str = "basement_gdopener/rssi";

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
    connected: Arc<AtomicBool>,
    needs_resubscribe: Arc<AtomicBool>,
    connection_thread_dead: bool,
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
        let connected = Arc::new(AtomicBool::new(false));
        let needs_resubscribe = Arc::new(AtomicBool::new(false));
        let connected_flag = connected.clone();
        let resubscribe_flag = needs_resubscribe.clone();

        let (client, mut connection) = EspMqttClient::new(mqtt_endpoint, &config)?;

        // The connection thread drives the MQTT state machine; it must be
        // running before we can subscribe or publish.
        std::thread::spawn(move || {
            while let Ok(event) = connection.next() {
                match event.payload() {
                    EventPayload::Connected(_) => {
                        log::info!("MQTT connected");
                        connected_flag.store(true, Ordering::Relaxed);
                        resubscribe_flag.store(true, Ordering::Relaxed);
                    }
                    EventPayload::Disconnected => {
                        log::warn!("MQTT disconnected");
                        connected_flag.store(false, Ordering::Relaxed);
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
            log::error!("MQTT connection thread exiting unexpectedly");
            connected_flag.store(false, Ordering::Relaxed);
        });

        // Return immediately — the underlying MQTT client handles reconnection
        // automatically (reconnect_timeout). The first Connected event will set
        // needs_resubscribe, and the main loop's resubscribe_if_needed() will
        // subscribe and publish state exactly as it does after any reconnect.
        // This avoids a hard failure if the broker isn't up at boot time.
        Ok(Self {
            client,
            command_rx,
            connected,
            needs_resubscribe,
            connection_thread_dead: false,
        })
    }

    /// Re-subscribes to the command topic after an MQTT reconnect.
    /// Returns true if a resubscription was performed.
    pub fn resubscribe_if_needed(&mut self) -> bool {
        if !self.needs_resubscribe.load(Ordering::Relaxed) {
            return false;
        }
        match self.client.subscribe(COMMAND_TOPIC, QoS::AtLeastOnce) {
            Ok(_) => {
                log::info!("Resubscribed to {}", COMMAND_TOPIC);
                self.needs_resubscribe.store(false, Ordering::Relaxed);
                true
            }
            Err(e) => {
                log::error!("Resubscribe failed: {:?}", e);
                false
            }
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    pub fn take_command(&mut self) -> Option<GarageCommand> {
        match self.command_rx.try_recv() {
            Ok(cmd) => Some(cmd),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                if !self.connection_thread_dead {
                    log::error!("MQTT connection thread has died");
                    self.connection_thread_dead = true;
                }
                None
            }
        }
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
            .publish(STATUS_TOPIC, QoS::AtMostOnce, false, b"online")
    }

    pub fn publish_rssi(&mut self, rssi: i8) -> Result<u32, EspError> {
        self.client.publish(
            RSSI_TOPIC,
            QoS::AtMostOnce,
            false,
            rssi.to_string().as_bytes(),
        )
    }

    pub fn publish_error(&mut self, error_msg: String) -> Result<u32, EspError> {
        self.client
            .publish(ERROR_TOPIC, QoS::AtLeastOnce, false, error_msg.as_bytes())
    }
}
