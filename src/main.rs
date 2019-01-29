extern crate mqtt;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::env;
use std::io::{Write};
use std::net::TcpStream;
use std::thread;
use std::time;

use mqtt::control::variable_header::ConnectReturnCode;
use mqtt::{Encodable, Decodable};
use mqtt::packet::*;
use mqtt::TopicName;

mod config;

fn main() {
     // configure logging
    env::set_var("RUST_LOG", env::var_os("RUST_LOG").unwrap_or_else(|| "info".into()));
    env_logger::init();

    // let config = match read_config() {
    //     Config(conf) => conf,
    //     None => write_default_config(),
    // };

    println!("Hello, world!");

    let server_addr = "some_server:1883";
    info!("Connecting to {:?} ... ", server_addr);
    let mut stream = TcpStream::connect(server_addr).unwrap();
    info!("Connected!");

    let client_id = "lsm_test_client";
    let user_name = "fakeuser".to_string();
    let password = "fakepass".to_string();
    info!("Client identifier {:?}", client_id);
    let mut conn = ConnectPacket::new("MQTT", client_id);
    conn.set_clean_session(true);
    conn.set_user_name(Some(user_name));
    conn.set_password(Some(password));
    let mut buf = Vec::new();
    conn.encode(&mut buf).unwrap();
    stream.write_all(&buf[..]).unwrap();

    let connack = ConnackPacket::decode(&mut stream).unwrap();
    info!("CONNACK {:?}", connack);

    if connack.connect_return_code() != ConnectReturnCode::ConnectionAccepted {
        panic!(
            "Failed to connect to server, return code {:?}",
            connack.connect_return_code()
        );
    }

    let mut cloned_stream = stream.try_clone().unwrap();
    thread::spawn(move || {
        loop {
            let packet = match VariablePacket::decode(&mut cloned_stream) {
                Ok(pk) => pk,
                Err(err) => {
                    error!("Error in receiving packet {:?}", err);
                    continue;
                }
            };
            trace!("PACKET {:?}", packet);

            match packet {
                VariablePacket::PingreqPacket(..) => {
                    let pingresp = PingrespPacket::new();
                    info!("Sending Ping response {:?}", pingresp);
                    pingresp.encode(&mut cloned_stream).unwrap();
                }
                VariablePacket::DisconnectPacket(..) => {
                    break;
                }
                _ => {
                    // Ignore other packets in pub client
                }
            }
        }
    });

    loop {
        // Create a new Publish packet
        let packet = PublishPacket::new(TopicName::new("mqtt/learning").unwrap(),
                                    QoSWithPacketIdentifier::Level0,
                                    b"Hello MQTT!".to_vec());
        let mut buf = Vec::new();
        packet.encode(&mut buf).unwrap();
        info!("Sent");
        // let puback = PubackPacket::decode(&mut stream).unwrap();
        // info!("PUBACK {:?}", puback);

        let duration = time::Duration::from_millis(1000);
        thread::sleep(duration);
    }
}
