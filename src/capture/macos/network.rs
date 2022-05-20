use super::appkit::{ns_string_to_string};
use objc::{
    class, msg_send,
    runtime::{Object},
    sel, sel_impl,
};

pub fn get_network_ssid() -> Option<String> {
    unsafe {
        let cw_wifi_client: *mut Object = msg_send![class!(CWWiFiClient), sharedWiFiClient];

        let interface: *mut Object = msg_send![cw_wifi_client, interface];

        if interface.is_null() {
            return None;
        }

        let ns_string: *mut Object = msg_send![interface, ssid];

        if ns_string.is_null() {
            return None;
        }

        ns_string_to_string(ns_string)
    }
}
