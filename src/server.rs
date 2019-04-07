use std::io::{Error, ErrorKind, Write};
use std::net::TcpStream;
use std::string::ToString;
use std::thread;

use crate::config::Config;

use mqtt::control::fixed_header::FixedHeaderError;
use mqtt::control::variable_header::ConnectReturnCode;
use mqtt::packet::*;
use mqtt::TopicName;
use mqtt::{Decodable, Encodable};

pub struct MQTTServer {
    server_addr: String,
    user_name: String,
    password: String,
    stream: Option<TcpStream>,
}

impl MQTTServer {
    pub fn connect(config: Config) -> Result<MQTTServer, Error> {
        let result = MQTTServer {
            server_addr: config.server_addr,
            user_name: config.user_name,
            password: config.password,
            stream: None,
        };
        result.try_reconnect()
    }

    pub fn reconnect(self) -> MQTTServer {
        match self.try_reconnect() {
            Ok(s) => s,
            Err(e) => {
                error!("Couldn't reconnect to broker! {:?}", e);
                MQTTServer {
                    server_addr: self.server_addr,
                    user_name: self.user_name,
                    password: self.password,
                    stream: None,
                }
            }
        }
    }

    pub fn try_reconnect(&self) -> Result<MQTTServer, Error> {
        info!("Connecting to {:?} ... ", self.server_addr);
        let mut stream = TcpStream::connect(self.server_addr.clone())?;
        info!("Connected!");

        let client_id = "lsm_test_client";
        info!("Client identifier {:?}", client_id);

        let mut conn = ConnectPacket::new("MQTT", client_id);
        conn.set_clean_session(true);
        conn.set_user_name(Some(self.user_name.clone()));
        conn.set_password(Some(self.password.clone()));

        let mut buf = Vec::new();
        match conn.encode(&mut buf) {
            Ok(k) => k,
            Err(error) => {
                error!("Could not encode Connection packet: {:?}", error);
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Could not encode Connection packet",
                ));
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
                return Err(Error::new(
                    ErrorKind::NotConnected,
                    "Unable to decode CONNACK packet",
                ));
            }
        };

        if connack.connect_return_code() != ConnectReturnCode::ConnectionAccepted {
            error!(
                "Failed to connect to server, return code {:?}",
                connack.connect_return_code()
            );
            return Err(Error::new(
                ErrorKind::NotConnected,
                "Failed to connect to server",
            ));
        }

        let server = MQTTServer {
            server_addr: self.server_addr.clone(),
            user_name: self.user_name.clone(),
            password: self.password.clone(),
            stream: Some(stream),
        };
        Ok(server)
    }

    pub fn start(self) -> Result<MQTTServer, String> {
        if let Some(stream) = &self.stream { 
            let mut cloned_stream = stream.try_clone().unwrap();
            thread::spawn(move || {
                loop {
                    let packet = match VariablePacket::decode(&mut cloned_stream) {
                        Ok(pk) => pk,
                        Err(error) => {
                            match handle_packet_receive_error(error) {
                                PacketReceiveError::ConnectionReset => {
                                    warn!("Receive thread terminating due to ConnectionReset!");
                                    break;
                                }
                                PacketReceiveError::Other => continue
                            }
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
        }

        Ok(self)
    }

    pub fn publish<M: Into<Vec<u8>>>(
        &mut self,
        topic: &str,
        message: M,
    ) -> Result<(), std::io::Error> {
        // Create a new Publish packet
        let mut packet = PublishPacket::new(
            TopicName::new(topic).unwrap(),
            QoSWithPacketIdentifier::Level0,
            message,
        );
        packet.set_retain(true);
        let mut buf = Vec::new();
        packet.encode(&mut buf).unwrap();
        match &self.stream {
            Some(stream) => {
                let mut stream = stream;
                stream.write_all(&buf[..])
            },
            None => Err(std::io::Error::new(ErrorKind::ConnectionReset, "Cannot send because TcpStream was reset and never reconnected!")),
        }
    }

    pub fn publish_value<V: ToString>(
        &mut self,
        topic: &str,
        value: V,
    ) -> Result<(), std::io::Error> {
        let message = value.to_string();
        self.publish(topic, message.as_bytes().to_vec())
    }
}

enum PacketReceiveError {
    ConnectionReset,
    Other,
}

fn handle_packet_receive_error(error: VariablePacketError) -> PacketReceiveError {
    match error {
        VariablePacketError::IoError(io_err) => {
            if let ErrorKind::ConnectionReset = io_err.kind() {
                warn!("Connection to MQTT broker was reset!");
                return PacketReceiveError::ConnectionReset;
            } else {
                error!("Unexpected IoError! {:?}", io_err);
            }
        }
        VariablePacketError::FixedHeaderError(header_err) => {
            if let FixedHeaderError::IoError(io_err) = header_err {
                if let ErrorKind::ConnectionReset = io_err.kind() {
                    warn!("Connection to MQTT broker was reset!");
                    return PacketReceiveError::ConnectionReset;
                } else {
                    error!("Unexpected IoError! {:?}", io_err);
                }
            } else {
                error!("Unexpected FixedHeaderError! {:?}", header_err);
            }
        }
        _ => {
            error!("Error in receiving packet {:?}", error);
        }
    }

    PacketReceiveError::Other
}
