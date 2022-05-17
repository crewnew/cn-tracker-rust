pub mod linux;
pub mod macos;
pub mod pc_common;
pub mod windows;

use std::thread;

#[cfg(target_os = "macos")]
pub fn create_capturer() -> Box<dyn Capturer> {
    Box::new(macos::appkit::MacOSCapturer::init())
}

#[cfg(target_os = "linux")]
pub fn capture_peripherals() {
    linux::peripherals::initiate_event_listeners().unwrap();
}
#[cfg(target_os = "macos")]
pub fn capture_peripherals() {
    thread::spawn(macos::peripherals::capture_peripherals);
}
#[cfg(target_os = "windows")]
pub fn capture_peripherals() {
    thread::spawn(windows::peripherals::capture_peripherals);
}

#[cfg(target_os = "windows")]
pub fn create_capturer() -> Box<dyn Capturer> {
    Box::new(windows::winwins::WindowsCapturer::init())
}

#[cfg(target_os = "linux")]
pub fn create_capturer() -> Box<dyn Capturer> {
    Box::new(linux::x11::init().unwrap())
}

pub trait Capturer: Send {
    fn capture(&mut self) -> anyhow::Result<pc_common::Event>;
}
