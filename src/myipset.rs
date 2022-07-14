use std::process::Command;

pub fn add(set: &str, ip: &str) {
    let _ = Command::new("ipset").args(["add", set, ip]).spawn();
}
