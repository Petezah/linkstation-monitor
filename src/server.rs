use std::io::{Write, Error, ErrorKind};
use std::net::TcpStream;
use std::thread;

use crate::config::Config;

use mqtt::control::variable_header::ConnectReturnCode;
use mqtt::{Encodable, Decodable};
use mqtt::packet::*;
use mqtt::TopicName;

pub struct MQTTServer {
    stream: TcpStream,
}

impl MQTTServer {
    pub fn connect(config: Config) -> Result<MQTTServer,Error> {
        info!("Connecting to {:?} ... ", config.server_addr);
        let mut stream = TcpStream::connect(config.server_addr)?;
        info!("Connected!");

        let client_id = "lsm_test_client";
        info!("Client identifier {:?}", client_id);

        let mut conn = ConnectPacket::new("MQTT", client_id);
        conn.set_clean_session(true);
        conn.set_user_name(Some(config.user_name));
        conn.set_password(Some(config.password));
        
        let mut buf = Vec::new();
        match conn.encode(&mut buf) {
            Ok(k) => k,
            Err(error) => {
                error!("Could not encode Connection packet: {:?}", error);
                return Err(Error::new(ErrorKind::InvalidData, "Could not encode Connection packet"));
            }
        };
        stream.write_all(&buf[..])?;

        let connack = match ConnackPacket::decode(&mut stream) {
            Ok(connack) => {
                info!("CONNACK {:?}", connack);
                connack
            }
            Err(error) => {
                error!("Unable to decode CONNACK packet! {:?}", error);
                return Err(Error::new(ErrorKind::NotConnected, "Unable to decode CONNACK packet"));
            }
        };

        if connack.connect_return_code() != ConnectReturnCode::ConnectionAccepted {
            error!(
                "Failed to connect to server, return code {:?}",
                connack.connect_return_code()
            );
            return Err(Error::new(ErrorKind::NotConnected, "Failed to connect to server"));
        }

        let server = MQTTServer {
            stream: stream
        };
        Ok(server)
    }

    pub fn start(self) -> Result<MQTTServer,String> {
        let mut cloned_stream = self.stream.try_clone().unwrap();
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

        let result = MQTTServer {
            stream: self.stream
        };
        Ok(result)
    }

    pub fn publish<M: Into<Vec<u8>>>(&mut self, topic: &str, message: M) -> Result<(),std::io::Error> {
        // Create a new Publish packet
        let packet = PublishPacket::new(TopicName::new(topic).unwrap(),
                                    QoSWithPacketIdentifier::Level0,
                                    message);
        let mut buf = Vec::new();
        packet.encode(&mut buf).unwrap();
        self.stream.write_all(&buf[..])
    }
}