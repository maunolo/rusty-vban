use std::io;
use std::io::Write;

pub fn info(msg: &str) {
    let msg = format!("[INFO] {}", msg);
    log(&msg);
}

pub fn warn(msg: &str) {
    let msg = format!("[WARN] {}", msg);
    log(&msg);
}

pub fn debug(msg: &str) {
    let msg = format!("[DEBUG] {}", msg);
    log(&msg);
}

pub fn error(msg: &str) {
    let msg = format!("[ERROR] {}", msg);
    log(&msg);
}

pub fn log(msg: &str) {
    let stdout = io::stdout();
    let _ = writeln!(&mut stdout.lock(), "{}", msg);
}
