extern crate toml;
extern crate serde;

use std::fmt::Debug;
use std::io;



#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Parse(toml::de::Error),
}

#[derive(Deserialize)]
pub struct Settings {
    pub mqtt: MQTT,
}

#[derive(Deserialize)]
pub struct MQTT {
    pub client_id: String,
    pub broker: String,
    pub broker_address: String,
    pub username: String,
    pub password: String,
    pub topic: String,
}

pub fn read_config<T: io::Read + Sized>(mut f: T) -> Result<Settings, ConfigError> {
    let mut buffer = String::new();
    try!(f.read_to_string(&mut buffer).map_err(ConfigError::Io));
    toml::from_str(&buffer).map_err(ConfigError::Parse)
}