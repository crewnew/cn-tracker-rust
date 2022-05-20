use nl80211::{parse_hex, parse_i8, parse_string, parse_u32, Socket};

// currently wifi only
// todo?:  get mac address of gateway (in ethernet this should be same as bssid of wifi)
pub fn get_network_ssid() -> Option<String> {
    let interfaces = Socket::connect().ok()?.get_interfaces_info().ok()?;

    for interface in interfaces {
        if let nl80211::Interface {
            ssid: Some(ssid),
            index: Some(_), // none if no wifi connected
            ..
        } = &interface
        {
            return Some(parse_string(&ssid));
        }
    }

    None
}
