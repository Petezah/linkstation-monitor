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
use std::io::ErrorKind;
use std::path::Path;
use std::thread;
use std::time;

mod config;
mod datatools;
mod server;

fn main() {
    // configure logging
    env::set_var(
        "RUST_LOG",
        env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()),
    );
    env_logger::init();

    // configure config location
    let config_root = env::var_os("LSMON_CONFIG_PATH").unwrap_or_else(|| "./".into());
    let config_root = Path::new(&config_root);
    let config_path = config_root.join("config.json");
    info!("Loading config from '{:?}'", config_path);

    let config = config::Config::read(config_path);
    let file_monitors = Vec::clone(&config.file_monitors);
    let mount_monitor = String::clone(&config.mount_monitor);
    let mount_monitor_topic = String::clone(&config.mount_monitor_topic);
    let publish_delay = config.publish_delay_ms;
    let server = match server::MQTTServer::connect(config) {
        Ok(s) => s,
        Err(error) => panic!("Couldn't connect to broker: {}", error),
    };

    let mut server = match server.start() {
        Ok(s) => s,
        Err(error) => panic!("Couldn't start server: {}", error),
    };

    let mut server_okay = true;
    loop {
        if !server_okay {
            info!("Connection was reset; trying to reestablish connection...");
            server = server.reconnect();
            server = match server.start() {
                Ok(s) => s,
                Err(error) => panic!("Couldn't restart server: {}", error),
            };
            server_okay = true;
        }

        match send_watches(&mut server, &file_monitors, &mount_monitor, &mount_monitor_topic) {
            Ok(_) => trace!("Sent all watches successfully!"),
            Err(error) => {
                if let ErrorKind::ConnectionReset = error.kind() {
                    warn!("Could not send watches!  Connection reset!");
                    server_okay = false;
                }
            }
        }

        let duration = time::Duration::from_millis(publish_delay);
        thread::sleep(duration);
    }
}

fn send_watches(
    server: &mut server::MQTTServer,
    file_monitors: &Vec<config::FileMonitor>,
    mount_monitor: &str,
    mount_monitor_topic: &str) -> Result<(), std::io::Error> {
    // Send file watches
    for mon in file_monitors {
        let value: f32 = match datatools::read_value_from_file(&mon.file, mon.index) {
            Some(v) => v,
            None => 0.0,
        };

        let value_str = value.to_string();
        match server.publish(&mon.topic, value_str.as_bytes().to_vec()) {
            Ok(_) => info!("Sent {} to {}", value_str, mon.topic),
            Err(error) => {
                warn!("Failed: {}", error); 
                return Err(error);
            },
        }
    }

    // Send filesystem info
    match datatools::FileSystemInfo::get() {
        Ok(fs_info) => {
            match fs_info
                .mounts
                .iter()
                .find(|info| info.mount == mount_monitor)
            {
                Some(array1_info) => {
                    let topic = format!("{}/used", mount_monitor_topic);
                    match server.publish_value(&topic, array1_info.used) {
                        Ok(_) => info!("Sent {} to {}", array1_info.used, topic),
                        Err(error) => {
                            warn!("Failed: {}", error); 
                            return Err(error);
                        },
                    }

                    let topic = format!("{}/size", mount_monitor_topic);
                    match server.publish_value(&topic, array1_info.size) {
                        Ok(_) => info!("Sent {} to {}", array1_info.size, topic),
                        Err(error) => {
                            warn!("Failed: {}", error); 
                            return Err(error);
                        },
                    }

                    let topic = format!("{}/usage", mount_monitor_topic);
                    match server.publish_value(&topic, array1_info.usage) {
                        Ok(_) => info!("Sent {} to {}", array1_info.usage, topic),
                        Err(error) => {
                            warn!("Failed: {}", error); 
                            return Err(error);
                        },
                    }
                }
                None => {
                    // Ignore
                }
            }
        }
        Err(error) => {
            warn!("Could not get fs info: {}", error); 
            // Do not return Err here; we only care about network errors
        },
    };

    Ok(())
}
