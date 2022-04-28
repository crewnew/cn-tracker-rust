// x11 capture types (must be cross-platform)

use serde::{Deserialize, Serialize};

use super::super::{Capturer, CapturerCreator};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct X11CaptureArgs {
    // captures from default screen
    /// if true, only capture the focused window.
    /// if false, capture all windows.
    pub only_focused_window: bool,
}

#[cfg(target_os = "linux")]
impl CapturerCreator for X11CaptureArgs {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>> {
        super::x11::init(self.clone()).map(|e| Box::new(e) as Box<dyn Capturer>)
    }
}

#[cfg(not(target_os = "linux"))]
impl CapturerCreator for X11CaptureArgs {
    fn create_capturer(&self) -> anyhow::Result<Box<dyn Capturer>> {
        anyhow::bail!("Not on Linux!")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WifiInterface {
    /// Interface essid
    pub ssid: String,
    /// Interface MAC address
    pub mac: String,
    /// Interface name (u8, String)
    pub name: String,
    /// Interface transmit power level in signed mBm units.
    pub power: u32,
    /// Signal strength average (i8, dBm)
    pub average_signal: i8,
    /// Station bssid (u8)
    pub bssid: String,
    /// Time since the station is last connected in seconds (u32)
    pub connected_time: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkInfo {
    pub wifi: Option<WifiInterface>,
}

// "2\u{0}4\u{0}5\u{0}6\u{0}8\u{0}9\u{0}1\u{0}" to array of strings
pub fn split_zero(s: &str) -> Vec<String> {
    let mut vec: Vec<String> = s.split('\0').map(String::from).collect();
    if vec.last().map(|e| e.is_empty()).unwrap_or(false) {
        // there seems to be an inconsistency:
        // the list in WM_CLASS is zero-terminated, as is the list in _NET_DESKTOP_NAMES on i3
        // but in bspwm it is not zero-terminated
        // https://github.com/phiresky/timetrackrs/issues/12
        vec.pop().unwrap();
    }
    vec
}
