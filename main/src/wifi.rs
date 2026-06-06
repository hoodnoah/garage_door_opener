use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{delay::FreeRtos, modem::Modem},
    nvs::EspDefaultNvsPartition,
    sys::{esp, esp_wifi_sta_get_ap_info, wifi_ap_record_t, EspError},
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};

pub struct WifiHandler<'a> {
    wifi: BlockingWifi<EspWifi<'a>>,
}

impl<'a> WifiHandler<'a> {
    pub fn new(modem: Modem, wifi_ssid: &str, wifi_password: &str) -> Result<Self, EspError> {
        let sys_loop = EspSystemEventLoop::take()?;
        let nvs = EspDefaultNvsPartition::take()?;

        let mut wifi = BlockingWifi::wrap(
            EspWifi::new(modem, sys_loop.clone(), Some(nvs))?,
            sys_loop.clone(),
        )?;

        wifi.set_configuration(&Configuration::Client(ClientConfiguration {
            ssid: wifi_ssid.try_into().unwrap(),
            password: wifi_password.try_into().unwrap(),
            auth_method: AuthMethod::WPA2Personal,
            ..Default::default()
        }))?;

        wifi.start()?;

        // ESP32-C3 Super Mini v1 has a broken antenna design; reducing TX power
        // avoids signal reflections that corrupt frames and cause auth failures.
        // 34 = 8.5 dBm in quarter-dBm units.
        // esp!(unsafe { esp_wifi_set_max_tx_power(34) })?;

        Ok(Self { wifi })
    }

    pub fn connect(&mut self) -> Result<(), EspError> {
        const MAX_RETRIES: u32 = 5;
        const RETRY_DELAY_MS: u32 = 2_000;

        for attempt in 1..=MAX_RETRIES {
            match self.wifi.connect() {
                Ok(_) => {
                    self.wifi.wait_netif_up()?;
                    return Ok(());
                }
                Err(e) if attempt < MAX_RETRIES => {
                    log::warn!(
                        "WiFi connect attempt {}/{} failed: {:?}, retrying...",
                        attempt,
                        MAX_RETRIES,
                        e
                    );
                    FreeRtos::delay_ms(RETRY_DELAY_MS);
                }
                Err(e) => return Err(e),
            }
        }
        unreachable!()
    }

    pub fn is_connected(&self) -> Result<bool, EspError> {
        self.wifi.is_connected()
    }

    pub fn rssi(&self) -> Result<i8, EspError> {
        let mut ap_info: wifi_ap_record_t = unsafe { std::mem::zeroed() };
        esp!(unsafe { esp_wifi_sta_get_ap_info(&mut ap_info) })?;
        Ok(ap_info.rssi)
    }

    pub fn ensure_connected(&mut self) -> Result<(), EspError> {
        if !self.is_connected()? {
            match self.connect() {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        } else {
            Ok(())
        }
    }
}
