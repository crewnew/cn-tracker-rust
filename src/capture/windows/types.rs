// windows capture types (must be cross-platform)
use super::super::{Capturer, CapturerCreator};

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowsCaptureArgs {}

#[cfg(windows)]
impl CapturerCreator for WindowsCaptureArgs {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>> {
        match super::winwins::WindowsCapturer::init() {
            Ok(e) => Ok(Box::new(e)),
            Err(e) => Err(e),
        }
    }
}
#[cfg(not(windows))]
impl CapturerCreator for WindowsCaptureArgs {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>> {
        anyhow::bail!("Not on Windows!")
    }
}
