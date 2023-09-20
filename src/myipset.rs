use std::process::Command;
use crate::config::CONFIG;
use log::{info, warn};

fn cmd_ret(args: Vec<&str>) -> bool {
    let p = Command::new("ipset").args(args).status().unwrap();
    p.success()
}

pub fn init() {
    let set = &CONFIG.ipset;
    let flush = cmd_ret(vec!["flush", set]);
    if flush {
        info!("flush ipset {} ok", set);
    } else {
        let create = cmd_ret(vec!["create", set, "hash:ip"]);
        if create {
            info!("create ipset {} ok", set);
        } else {
            panic!("failed to create ipset {}", set);
        }
    }
}

pub fn add(ip: &str) {
    let set = &CONFIG.ipset;
    let p = cmd_ret(vec!["add", set, ip]);
    if !p {
        warn!("failed to add ip {} to ipset {}", ip, set);
    }
}
