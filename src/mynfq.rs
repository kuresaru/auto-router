use std::thread::JoinHandle;
use std::{os::raw::c_int, thread};
use crate::rdsutils;
use crate::config::CONFIG;

pub type ProcessCallback = Option<unsafe extern "C" fn(ack: u8, ip: u32)>;

extern "C" {
    pub fn run_nfq(qid: u16) -> c_int;
    pub fn set_process_cb(cb: ProcessCallback);
}

pub extern "C" fn cb(ack: u8, ip: u32) {
    if ack == 0 {
        rdsutils::mark_syn(&ip);
    } else {
        rdsutils::mark_ack(&ip);
    }
}

pub fn start() -> JoinHandle<()> {
    unsafe { set_process_cb(Some(cb)); }
    thread::spawn(||unsafe { run_nfq(CONFIG.nfq_id); })
}