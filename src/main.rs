use log::info;

#[macro_use]
extern crate lazy_static;
extern crate log;
extern crate simple_logger;

mod myipset;
mod rdsutils;
mod mynfq;
mod config;

fn main() {
    simple_logger::init().unwrap();
    myipset::init();
    let rcb_th = rdsutils::start();
    let nfq_th = mynfq::start();
    info!("auto-router started");
    rcb_th.join().unwrap();
    nfq_th.join().unwrap();
}
