extern crate mqtt;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate serde;
#[macro_use]
extern crate serde_derive;
//#[macro_use]
extern crate serde_json;

use std::env;
use std::thread;
use std::time;

mod config;
mod server;

fn main() {
     // configure logging
    env::set_var("RUST_LOG", env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()));
    env_logger::init();

    let config = config::Config::read("config.json");
    let server = match server::MQTTServer::connect(config) {
        Ok(s) => s,
        Err(error) => panic!("Couldn't connect to broker: {}", error),
    };

    let mut server = match server.start() {
        Ok(s) => s,
        Err(error) => panic!("Couldn't start server: {}", error),
    };

    loop {
        match server.publish("mqtt/learning", b"Hello MQTT!".to_vec()) {
            Ok(_) => info!("Sent"),
            Err(error) => warn!("Failed: {}", error),
        }

        let duration = time::Duration::from_millis(1000);
        thread::sleep(duration);
    }
}
