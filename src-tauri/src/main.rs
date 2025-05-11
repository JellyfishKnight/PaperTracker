// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod serial;

use serial::esp32::start_serial_mod;

fn main() {
    start_serial_mod();
    papertracker_lib::run();
}
