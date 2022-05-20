use regex::Regex;
use std::{os::windows::process::CommandExt, process::Command};

pub fn get_network_ssid() -> Option<String> {
    lazy_static! {
        static ref SSID_MATCH: Regex = Regex::new(r"(?m)^\s*SSID\s*:\s*(.*?)\r?$").unwrap();
    }
    let output = Command::new("netsh")
        .creation_flags(winapi::um::winbase::CREATE_NO_WINDOW)
        .args(&["wlan", "show", "interfaces"])
        .output()
        .ok()?;
    let output = String::from_utf8_lossy(&output.stdout);
    let matched = SSID_MATCH
        .captures(&output)
        .map(|m| m.get(1).unwrap().as_str().to_string());
    matched
}
