use std::env;

fn main() {
    // load .env file for build
    dotenvy::dotenv().ok();

    // re-export as cargo env vars
    if let Ok(ssid) = env::var("WIFI_SSID") {
        println!("cargo:rustc-env=WIFI_SSID={}", ssid);
    }

    if let Ok(wifi_pass) = env::var("WIFI_PASSWORD") {
        println!("cargo:rustc-env=WIFI_PASSWORD={}", wifi_pass);
    }

    if let Ok(mqtt_endpoint) = env::var("MQTT_ENDPOINT") {
        println!("cargo:rustc-env=MQTT_ENDPOINT={}", mqtt_endpoint);
    }

    if let Ok(mqtt_user) = env::var("MQTT_USER") {
        println!("cargo:rustc-env=MQTT_USER={}", mqtt_user);
    }

    if let Ok(mqtt_pass) = env::var("MQTT_PASS") {
        println!("cargo:rustc-env=MQTT_PASS={}", mqtt_pass);
    }

    println!("cargo:rerun-if-changed=.env");

    // embuild stuff
    embuild::espidf::sysenv::output();
}
