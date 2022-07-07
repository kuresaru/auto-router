use std::os::raw::c_int;

pub type ProcessCallback = Option<unsafe extern "C" fn(payload: *mut u8, len: c_int)>;

extern "C" {
    pub fn run_main() -> c_int;
    pub fn set_process_cb(cb: ProcessCallback);
}

pub extern "C" fn cb(_payload: *mut u8, len: c_int) {
    println!("got packet len {}", len);
}

fn main() {
    unsafe {
        set_process_cb(Some(cb));
        run_main();
    }
}
