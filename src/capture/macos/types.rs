// MacOS capture types (must be cross-platform)
use super::super::{Capturer, CapturerCreator};

#[derive(Debug, Serialize, Deserialize)]
pub struct MacOSCaptureArgs {}

#[cfg(target_os = "macos")]
impl CapturerCreator for MacOSCaptureArgs {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>> {
        Ok(Box::new(super::appkit::MacOSCapturer::init()))
    }
}

#[cfg(not(target_os = "macos"))]
impl CapturerCreator for MacOSCaptureArgs {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>> {
        anyhow::bail!("Not on MacOS!")
    }
}
