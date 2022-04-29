pub mod linux;
pub mod macos;
pub mod pc_common;
pub mod windows;

use std::time::Duration;

#[enum_dispatch]
#[derive(Debug, Serialize, Deserialize)]
pub enum CaptureArgs {
    NativeDefault(NativeDefaultArgs),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NativeDefaultArgs {}

fn default_capture_args() -> CaptureArgs {
    CaptureArgs::NativeDefault(NativeDefaultArgs {})
}

#[cfg(target_os = "macos")]
pub fn create_capturer() -> Box<dyn Capturer> {
   Box::new(macos::appkit::MacOSCapturer::init())
}

#[cfg(target_os = "windows")]
pub fn create_capturer() -> Box<dyn Capturer> {
    Box::new(windows::winwins::WindowsCapturer::init())
}

#[cfg(target_os = "linux")]
pub fn create_capturer() -> Box<dyn Capturer> {
    Box::new(linux::x11::init().unwrap())
}

impl CapturerCreator for NativeDefaultArgs {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>> {
        default_capture_args().create_capturer()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CaptureConfig {
    pub interval: Duration,
    pub args: CaptureArgs,
}

#[enum_dispatch(CaptureArgs)]
pub trait CapturerCreator {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>>;
}

pub trait Capturer: Send {
    fn capture(&mut self) -> anyhow::Result<pc_common::Event>;
}
