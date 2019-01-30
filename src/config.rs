use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub server_addr: String,
    pub user_name: String,
    pub password: String,
    pub file_monitors: Vec<FileMonitor>,
    pub mount_monitor: String,
    pub mount_monitor_topic: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileMonitor {
    pub topic: String,
    pub file: String,
    pub index: usize
}

impl Config {
    pub fn make_default() -> Config {
        Config {
            server_addr: String::from("127.0.0.1:1883"),
            user_name: String::from("user"),
            password: String::from("password"),
            file_monitors: vec![
                FileMonitor {
                    topic: String::from("mqtt/learning"),
                    file: String::from("/proc/uptime"),
                    index: 0
                }
                ],
            mount_monitor: String::from("/mnt/c"),
            mount_monitor_topic: String::from("mqtt/mount_monitor")
        }
    }

    pub fn read<P: AsRef<Path>>(path: P) -> Config {
        let config_file = read_config_file(path);
        let config: Config = match serde_json::from_str(&config_file){
            Ok(c) => c,
            Err(error) => {
                panic!("Could not read config file!  {}", error);
            },
        };
        config
    }
}

fn read_config_file<P: AsRef<Path>>(path: P) -> String {
    let mut config_file = get_config_file(path);
    let mut string = String::new();
    config_file.read_to_string(&mut string)
        .expect("Could not read config file!");
    string
}

fn get_config_file<P: AsRef<Path>>(path: P) -> File {
    let file = File::open(&path);
    match file {
        Ok(file) => file,
        Err(error) => {
            warn!("Config file did not exist! {}  Creating default config", error);
            make_default_config_file(path)
        },
    }
}

fn make_default_config_file<P: AsRef<Path>>(path: P) -> File {
    let default_config = Config::make_default();
    let default_config = serde_json::to_string(&default_config)
        .expect("Failed to make default configuration!");

    fs::write(&path, default_config)
        .expect("Failed to write default config file!");

    File::open(path)
        .expect("Failed to reopen default config file!")
}
