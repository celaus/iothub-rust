extern crate mqtt;
extern crate native_tls;
extern crate clap;
#[macro_use]
extern crate serde_derive;
mod config;

use native_tls::{TlsConnector, TlsStream};
use std::io::Write;
use std::net::TcpStream;
use std::fs::File;
use std::thread;
use std::time::Duration;
use mqtt::{Encodable, Decodable};
use mqtt::packet::*;
use mqtt::{TopicName};
use mqtt::control::variable_header::ConnectReturnCode;
use clap::{Arg, App};
use config::read_config;


fn connect(broker: String,
           username: String,
           password: String,
           client_id: String,
           verify_name: String)
           -> TlsStream<TcpStream> {
    let connector = TlsConnector::builder().unwrap().build().unwrap();
    let stream = TcpStream::connect(&broker).unwrap();
    let mut stream = connector.connect(&verify_name, stream).unwrap();
    let mut conn = ConnectPacket::new("MQTT", client_id.as_ref());

    conn.set_clean_session(true);
    conn.set_user_name(Some(username).to_owned());
    conn.set_password(Some(password.to_owned()));
    conn.set_client_identifier(client_id);

    let mut buf = Vec::new();
    conn.encode(&mut buf).unwrap();
    stream.write_all(&buf[..]).unwrap();
    let connack = ConnackPacket::decode(&mut stream).unwrap();
    if connack.connect_return_code() != ConnectReturnCode::ConnectionAccepted {
        panic!("Failed to connect to server, return code {:?}",
               connack.connect_return_code());
    }
    return stream;
}

fn publish(stream: &mut TlsStream<TcpStream>, msg: String, topic: TopicName) {
    let packet = PublishPacket::new(topic, QoSWithPacketIdentifier::Level1(10), msg);
    let mut buf = Vec::new();
    packet.encode(&mut buf).unwrap();
    stream.write_all(&buf).unwrap();
}

fn main() {

    let matches = App::new("MQTT publisher")
        .version("0.2.0")
        .author("Claus Matzinger. <claus.matzinger+kb@gmail.com>")
        .about("Sends data to an MQTT broker")
        .arg(Arg::with_name("config")
                 .short("c")
                 .long("config")
                 .help("Sets a custom config file [default: config.toml]")
                 .value_name("config.toml")
                 .takes_value(true))
        .get_matches();
    let config_filename = matches.value_of("config").unwrap_or("config.toml");
    let mut f = File::open(config_filename)
        .expect(&format!("Can't open configuration file: {}", config_filename));
    let settings = read_config(&mut f).expect("Can't read configuration file.");
    println!("Connecting to mqtts://{}", settings.mqtt.broker_address);

    let topic_name = TopicName::new(settings.mqtt.topic.clone()).unwrap();

    let mut stream = connect(settings.mqtt.broker_address,
                             settings.mqtt.username,
                             settings.mqtt.password,
                             settings.mqtt.client_id,
                             settings.mqtt.broker);

    let mut i = 0;
    loop {
        i += 1;
        let msg = format!("{}", i);
        println!("Sending message '{}' to topic: '{}'",
                 msg,
                 settings.mqtt.topic);
        publish(&mut stream, msg, topic_name.clone());
        thread::sleep(Duration::from_millis(3000));
    }
}
