extern crate mqtt;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate scan_fmt;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::env;
use std::thread;
use std::time;

mod config;
mod datatools;
mod server;

fn main() {
     // configure logging
    env::set_var("RUST_LOG", env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()));
    env_logger::init();

    let config = config::Config::read("config.json");
    let file_monitors = Vec::clone(&config.file_monitors);
    let mount_monitor = String::clone(&config.mount_monitor);
    let mount_monitor_topic = String::clone(&config.mount_monitor_topic);
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
            let value: f32 = match datatools::read_value_from_file(&mon.file, mon.index) {
                Some(v) => v,
                None => 0.0
            };

            let value_str = value.to_string();
            match server.publish(&mon.topic, value_str.as_bytes().to_vec()) {
                Ok(_) => info!("Sent {} to {}", value_str, mon.topic),
                Err(error) => warn!("Failed: {}", error),
            }
        }

        // Send filesystem info
        match datatools::FileSystemInfo::get() {
            Ok(fs_info) => {
                match fs_info.mounts.iter().find(|info| info.mount == mount_monitor) {
                    Some(array1_info) => {
                        let topic = format!("{}/used", mount_monitor_topic);
                        match server.publish_value(&topic, array1_info.used) {
                            Ok(_) => info!("Sent {} to {}", array1_info.used, topic),
                            Err(error) => warn!("Failed: {}", error),  
                        }

                        let topic = format!("{}/size", mount_monitor_topic);
                        match server.publish_value(&topic, array1_info.size) {
                            Ok(_) => info!("Sent {} to {}", array1_info.size, topic),
                            Err(error) => warn!("Failed: {}", error),  
                        }

                        let topic = format!("{}/usage", mount_monitor_topic);
                        match server.publish_value(&topic, array1_info.usage) {
                            Ok(_) => info!("Sent {} to {}", array1_info.usage, topic),
                            Err(error) => warn!("Failed: {}", error),  
                        }
                    }
                    None => {
                        // Ignore
                    }
                }
            }
            Err(error) => warn!("Could not get fs info: {}", error),
        };

        let duration = time::Duration::from_millis(5000);
        thread::sleep(duration);
    }
}
