use std::{fs::File, io::Read};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub redis_server: String,
    pub syn_expire: String,
    pub ack_expire: String,
    pub ipset: String,
    pub nfq_id: u16,
}

lazy_static! {
    pub static ref CONFIG: Config = {
        let mut cfg_file = File::open("config.yml").expect("Failed to open config.yml");
        let mut cfg_text = String::new();
        cfg_file
            .read_to_string(&mut cfg_text)
            .expect("Failed to read config.yml");
        let cfg: Config = serde_yaml::from_str(&cfg_text).expect("Config error");
        cfg
    };
}