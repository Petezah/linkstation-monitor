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
mod filetools;
mod server;

fn main() {
     // configure logging
    env::set_var("RUST_LOG", env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()));
    env_logger::init();

    let config = config::Config::read("config.json");
    let file_monitors = Vec::clone(&config.file_monitors);
    let server = match server::MQTTServer::connect(config) {
        Ok(s) => s,
        Err(error) => panic!("Couldn't connect to broker: {}", error),
    };

    let mut server = match server.start() {
        Ok(s) => s,
        Err(error) => panic!("Couldn't start server: {}", error),
    };

    loop {
        // Send file watches
        for mon in &file_monitors {
            let value: f32 = match filetools::read_value_from_file(&mon.file, mon.index) {
                Some(v) => v,
                None => 0.0
            };

            let value_str = value.to_string();
            match server.publish(&mon.topic, value_str.as_bytes().to_vec()) {
                Ok(_) => info!("Sent {} to {}", value_str, mon.topic),
                Err(error) => warn!("Failed: {}", error),
            }
        }

        let duration = time::Duration::from_millis(5000);
        thread::sleep(duration);
    }
}
