use std::thread;
use std::thread::JoinHandle;
use log::info;
use redis::PubSubCommands;
use crate::myipset;
use crate::config::CONFIG;

static mut RDS_CONN_COMMAND: Option<redis::Connection> = None;
static mut RDS_CONN_EVENT: Option<redis::Connection> = None;

pub fn mark_syn(ip: &u32) {
    let con = unsafe { RDS_CONN_COMMAND.as_mut().unwrap() };
    let syn_expire = &CONFIG.syn_expire;
    let key_syn = format!("autorouter:syn_{:08x}", ip);
    let key_ack = format!("autorouter:ack_{:08x}", ip);
    let mut exists: bool = redis::cmd("EXISTS").arg(&key_ack).query(con).unwrap();
    if !exists {
        exists = redis::cmd("EXISTS").arg(&key_syn).query(con).unwrap();
        if !exists {
            let _: () = redis::cmd("SETEX")
                .arg(&[&key_syn, syn_expire, "syn"])
                .query(con)
                .unwrap();
        }
    }
}

pub fn mark_ack(ip: &u32) {
    let con = unsafe { RDS_CONN_COMMAND.as_mut().unwrap() };
    let ack_expire = &CONFIG.ack_expire;
    let key_syn = format!("autorouter:syn_{:08x}", ip);
    let key_ack = format!("autorouter:ack_{:08x}", ip);
    let _: () = redis::cmd("DEL").arg(&key_syn).query(con).unwrap();
    let _: () = redis::cmd("SETEX")
        .arg(&[&key_ack, ack_expire, "ack"])
        .query(con)
        .unwrap();
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
                myipset::add(&ip);
                info!("add ip {} to ipset", &ip);
            }
        }
    }
    redis::ControlFlow::Continue
}

fn rcb_thread() {
    let con;
    unsafe {
        con = RDS_CONN_EVENT.as_mut().unwrap(); // FIXME: app will panic when connection lost.
    }
    con.psubscribe("__keyevent@0__:expired", exp_cb).unwrap();
}

pub fn start() -> JoinHandle<()> {
    let redis_server: &str = CONFIG.redis_server.as_str();
    let client = redis::Client::open(redis_server).unwrap();
    unsafe {
        RDS_CONN_COMMAND = Some(client.get_connection().unwrap());
        RDS_CONN_EVENT = Some(client.get_connection().unwrap());
    }
    thread::spawn(rcb_thread)
}