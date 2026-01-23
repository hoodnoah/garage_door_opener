use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{delay::FreeRtos, modem::Modem},
    nvs::EspDefaultNvsPartition,
    sys::EspError,
    wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi},
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

        wifi.start()?;

        wifi.set_configuration(&Configuration::Client(ClientConfiguration {
            ssid: wifi_ssid.try_into().unwrap(),
            password: wifi_password.try_into().unwrap(),
            ..Default::default()
        }))?;

        Ok(Self { wifi: wifi })
    }

    pub fn connect(&mut self) -> Result<(), EspError> {
        self.wifi.connect()?;
        Ok(FreeRtos::delay_ms(10000))
    }
}
