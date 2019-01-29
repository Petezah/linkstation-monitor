use std::fs;
use std::fs::File;
use std::io::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    server_addr: String,
    user_name: String,
    password: String
}

impl Config {
    pub fn make_default() -> Config {
        Config {
            server_addr: String::from("127.0.0.1:1883"),
            user_name: String::from("user"),
            password: String::from("password")
        }
    }
}

pub fn read_config() -> Config {
    let config_file = read_config_file();
    let config: Config = match serde_json::from_str(&config_file){
        Ok(c) => c,
        Err(error) => {
            panic!("Could not read config file!  {}", error);
        },
    };
    config
}

fn read_config_file() -> String {
    let mut config_file = get_config_file();
    let mut string = String::new();
    config_file.read_to_string(&mut string)
        .expect("Could not read config file!");
    string
}

fn get_config_file() -> File {
    let file = File::open("config.json");
    match file {
        Ok(file) => file,
        Err(error) => {
            warn!("Config file did not exist! {}  Creating default config", error);
            make_default_config_file()
        },
    }
}

fn make_default_config_file() -> File {
    let default_config = Config::make_default();
    let default_config = serde_json::to_string(&default_config)
        .expect("Failed to make default configuration!");

    fs::write("config.json", default_config)
        .expect("Failed to write default config file!");

    File::open("config.json")
        .expect("Failed to reopen default config file!")
}
