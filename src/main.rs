use std::{os::raw::c_int, thread};

use redis::PubSubCommands;

mod myipset;

pub type ProcessCallback = Option<unsafe extern "C" fn(ack: u8, ip: u32)>;

extern "C" {
    pub fn run_nfq() -> c_int;
    pub fn set_process_cb(cb: ProcessCallback);
}

const REDIS_SERVER: &str = "redis://192.168.6.1/";
const SYN_EX: &str = "3";
const ACK_EX: &str = "86400";
const IPSET: &str = "proxy1";

static mut RDS_CONN: Option<redis::Connection> = None;
static mut RDS_CONN_PS: Option<redis::Connection> = None;

fn mark_syn(con: &mut redis::Connection, key_syn: &String, key_ack: &String) {
    let mut exists: bool = redis::cmd("EXISTS").arg(key_ack).query(con).unwrap();
    if !exists {
        exists = redis::cmd("EXISTS").arg(key_syn).query(con).unwrap();
        if !exists {
            println!("{}", key_syn);
            let _: () = redis::cmd("SETEX")
                .arg(&[key_syn, SYN_EX, "syn"])
                .query(con)
                .unwrap();
        }
    }
}

fn mark_ack(con: &mut redis::Connection, key_syn: &String, key_ack: &String) {
    println!("{}", key_ack);
    let _: () = redis::cmd("DEL").arg(key_syn).query(con).unwrap();
    let _: () = redis::cmd("SETEX")
        .arg(&[key_ack, ACK_EX, "ack"])
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
                myipset::add(IPSET, &ip);
                println!("{} expired, add {} to {}", &payload, &ip, IPSET);
            }
        }
    }
    redis::ControlFlow::Continue
}

fn nfq_thread() {
    unsafe {
        run_nfq();
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
    let client = redis::Client::open(REDIS_SERVER).unwrap();
    unsafe {
        RDS_CONN = Some(client.get_connection().unwrap());
        RDS_CONN_PS = Some(client.get_connection().unwrap());
        set_process_cb(Some(cb));
    }
    let _rcb_th = thread::spawn(rcb_thread);
    // let nfq_th = thread::spawn(nfq_thread);
    // nfq_th.join().unwrap();
    println!("nfq started");
    nfq_thread();
}
