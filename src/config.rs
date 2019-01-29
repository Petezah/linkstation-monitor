use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

#[derive(Serialize, Deserialize)]
struct Config {
    server_addr: String,
    user_name: String,
    password: String
}

fn read_config() -> Option<Config> {
    let config_file = fs::read_to_string("config.json");
    let config_file = match config_file {
        Ok(contents) => contents,
        Err(error) => {
            warn!("Default config 'config.json' does not exist");
            None
        },
    };

    let config: Config = match serde_json::from_str(&config_file){
        Config(c) => {
            Some(config)
            },
            Err => {
                None
            },
        };

    config_file
}

fn write_default_config() -> Config {
    let default_config = Config {
        server_addr: String::from("127.0.0.1:1883"),
        user_name: String::from("user"),
        password: String::from("password")
    };

    default_config
}
