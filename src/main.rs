#[macro_use]
extern crate lazy_static;

use std::{fs::File, io::Read, os::raw::c_int, thread};

use redis::PubSubCommands;
use serde::Deserialize;

mod myipset;

pub type ProcessCallback = Option<unsafe extern "C" fn(ack: u8, ip: u32)>;

extern "C" {
    pub fn run_nfq(qid: u16) -> c_int;
    pub fn set_process_cb(cb: ProcessCallback);
}

#[derive(Deserialize)]
struct Config {
    redis_server: String,
    syn_expire: String,
    ack_expire: String,
    ipset: String,
    nfq_id: u16,
}

lazy_static! {
    static ref CONFIG: Config = {
        let mut cfg_file = File::open("config.yml").expect("Failed to open config.yml");
        let mut cfg_text = String::new();
        cfg_file
            .read_to_string(&mut cfg_text)
            .expect("Failed to read config.yml");
        let cfg: Config = serde_yaml::from_str(&cfg_text).expect("Config error");
        cfg
    };
}

static mut RDS_CONN: Option<redis::Connection> = None;
static mut RDS_CONN_PS: Option<redis::Connection> = None;

fn mark_syn(con: &mut redis::Connection, key_syn: &String, key_ack: &String) {
    let mut exists: bool = redis::cmd("EXISTS").arg(key_ack).query(con).unwrap();
    if !exists {
        exists = redis::cmd("EXISTS").arg(key_syn).query(con).unwrap();
        if !exists {
            println!("{}", key_syn);
            let _: () = redis::cmd("SETEX")
                .arg(&[key_syn, &CONFIG.syn_expire, "syn"])
                .query(con)
                .unwrap();
        }
    }
}

fn mark_ack(con: &mut redis::Connection, key_syn: &String, key_ack: &String) {
    println!("{}", key_ack);
    let _: () = redis::cmd("DEL").arg(key_syn).query(con).unwrap();
    let _: () = redis::cmd("SETEX")
        .arg(&[key_ack, &CONFIG.ack_expire, "ack"])
        .query(con)
        .unwrap();
}

pub extern "C" fn cb(ack: u8, ip: u32) {
    let con;
    unsafe {
        con = RDS_CONN.as_mut().unwrap(); // FIXME: app will panic when connection lost.
    }
    let key_syn = format!("autorouter:syn_{:08x}", ip);
    let key_ack = format!("autorouter:ack_{:08x}", ip);
    if ack == 0 {
        mark_syn(con, &key_syn, &key_ack);
    } else {
        mark_ack(con, &key_syn, &key_ack);
    }
}

fn exp_cb(msg: redis::Msg) -> redis::ControlFlow<()> {
    let payload: Result<String, redis::RedisError> = msg.get_payload();
    if payload.is_ok() {
        let payload = payload.unwrap();
        if payload.len() == 23 && payload.starts_with("autorouter:syn_") {
            let ip = &payload[15..23];
            let ip = u32::from_str_radix(ip, 16);
            if ip.is_ok() {
                let ip = ip.unwrap();
                let ip = format!(
                    "{}.{}.{}.{}",
                    (ip >> 24) as u8,
                    (ip >> 16) as u8,
                    (ip >> 8) as u8,
                    ip as u8
                );
                myipset::add(&CONFIG.ipset, &ip);
                println!("{} expired, add {} to {}", &payload, &ip, &CONFIG.ipset);
            }
        }
    }
    redis::ControlFlow::Continue
}

fn nfq_thread() {
    unsafe {
        run_nfq(CONFIG.nfq_id);
    }
}

fn rcb_thread() {
    let con;
    unsafe {
        con = RDS_CONN_PS.as_mut().unwrap(); // FIXME: app will panic when connection lost.
    }
    con.psubscribe("__keyevent@0__:expired", exp_cb).unwrap();
}

fn main() {
    let redis_server: &str = CONFIG.redis_server.as_str();
    let client = redis::Client::open(redis_server).unwrap();
    unsafe {
        RDS_CONN = Some(client.get_connection().unwrap());
        RDS_CONN_PS = Some(client.get_connection().unwrap());
        set_process_cb(Some(cb));
    }
    let rcb_th = thread::spawn(rcb_thread);
    let nfq_th = thread::spawn(nfq_thread);
    rcb_th.join().unwrap();
    nfq_th.join().unwrap();
}
