use super::{
    super::{
        pc_common::{Event, Process, Window},
        Capturer,
    },
    peripherals::capture_peripherals,
};

use accessibility_sys::{
    kAXErrorSuccess, kAXFocusedWindowAttribute, kAXTitleAttribute, kAXTrustedCheckOptionPrompt,
    AXIsProcessTrustedWithOptions, AXUIElementCopyAttributeValue, AXUIElementCreateApplication,
};
use anyhow::Context;
use core_foundation::{
    array::CFArray,
    base::{CFRelease, FromVoid, ItemRef, TCFType, ToVoid},
    dictionary::{CFDictionary, CFMutableDictionary},
    number::CFNumber,
    string::{kCFStringEncodingUTF8, CFString, CFStringGetCStringPtr, CFStringRef},
};
use core_graphics::window::{
    kCGNullWindowID, kCGWindowListOptionOnScreenOnly, CGWindowListCopyWindowInfo,
};
use objc::{class, msg_send, runtime::Object, sel, sel_impl};
use std::{
    sync::atomic::{AtomicI32, Ordering},
    time::Duration,
};

use std::{
    ffi::{c_void, CStr},
    thread,
};
use sysinfo::{Pid, System, SystemExt};

static FRONTMOST_APPLICATION_PID: AtomicI32 = AtomicI32::new(0);

pub struct MacOSCapturer {
    accessibility_permission: bool,
    system: System,
}

impl MacOSCapturer {
    pub fn init() -> MacOSCapturer {
        let accessibility_permission = unsafe { check_accessibility_permission() };

        thread::spawn(capture_peripherals);

        MacOSCapturer {
            accessibility_permission,
            system: System::new(),
        }
    }

    pub unsafe fn get_focused_window(&mut self) -> Option<Window> {
        let pid = FRONTMOST_APPLICATION_PID.load(Ordering::Relaxed);

        debug!("Frontmost Application PID: {}", pid);

        let sysinfo_pid = Pid::from(pid);

        self.system.refresh_process(sysinfo_pid);

        let process: Process = self.system.process(sysinfo_pid)?.into();

        let app_ref = AXUIElementCreateApplication(pid);

        let mut title: Option<String> = None;

        let mut ax_element_ref: *const c_void = std::ptr::null();

        if AXUIElementCopyAttributeValue(
            app_ref,
            CFString::from_static_string(kAXFocusedWindowAttribute).as_concrete_TypeRef(),
            &mut ax_element_ref,
        ) == kAXErrorSuccess
        {
            let mut cf_string: *const c_void = std::ptr::null();

            if AXUIElementCopyAttributeValue(
                ax_element_ref as *mut _,
                CFString::from_static_string(kAXTitleAttribute).as_concrete_TypeRef(),
                &mut cf_string,
            ) == kAXErrorSuccess
            {
                let string = CFString::from_void(cf_string).to_string();

                title = Some(string);

                CFRelease(cf_string);
            }
            CFRelease(ax_element_ref);
        }

        CFRelease(app_ref as *const _);

        Some(Window { title, process })
    }

    /// Gets all currently running apps that may have UIs and are visible in the dock.
    /// Reference: https://developer.apple.com/documentation/appkit/nsapplicationactivationpolicy?language=objc
    pub unsafe fn get_windows(&mut self) -> Vec<Window> {
        let MacOSCapturer {
            accessibility_permission,
            ..
        } = *self;

        let mut windows: Vec<Window> = vec![];

        let cf_array: ItemRef<CFArray<CFDictionary<CFStringRef, *const c_void>>> =
            CFArray::from_void(CGWindowListCopyWindowInfo(
                kCGWindowListOptionOnScreenOnly,
                kCGNullWindowID,
            ) as *const _);

        for window in cf_array.iter() {
            let (keys, values) = window.get_keys_and_values();

            let mut pid: Option<i32> = None;

            for i in 0..keys.len() {
                let key = CFStringGetCStringPtr(keys[i] as _, kCFStringEncodingUTF8);

                let key = CStr::from_ptr(key).to_str().unwrap();

                match key {
                    "kCGWindowOwnerPID" => {
                        pid = CFNumber::from_void(values[i]).to_i32();
                    }
                    _ => (),
                };
            }

            if let Some(pid) = pid {
                let sysinfo_pid = Pid::from(pid);
                self.system.refresh_process(sysinfo_pid);
                if let Some(process) = self.system.process(sysinfo_pid) {
                    let mut title: Option<String> = None;

                    if accessibility_permission {
                        let app_ref = AXUIElementCreateApplication(pid);

                        let mut ax_element_ref: *const c_void = std::ptr::null();

                        if AXUIElementCopyAttributeValue(
                            app_ref,
                            CFString::from_static_string(kAXFocusedWindowAttribute)
                                .as_concrete_TypeRef(),
                            &mut ax_element_ref,
                        ) == kAXErrorSuccess
                        {
                            let mut cf_string: *const c_void = std::ptr::null();

                            if AXUIElementCopyAttributeValue(
                                ax_element_ref as *mut _,
                                CFString::from_static_string(kAXTitleAttribute)
                                    .as_concrete_TypeRef(),
                                &mut cf_string,
                            ) == kAXErrorSuccess
                            {
                                let string = CFString::from_void(cf_string).to_string();

                                title = Some(string);

                                CFRelease(cf_string);
                            }
                            CFRelease(ax_element_ref);
                        }
                        CFRelease(app_ref as *const _);
                    }

                    let macos_window = Window {
                        title,
                        process: process.into(),
                    };

                    windows.push(macos_window);
                }
            }
        }

        windows
    }
}

impl Capturer for MacOSCapturer {
    fn capture(&mut self) -> anyhow::Result<Event> {
        let mut windows = Vec::with_capacity(1);

        unsafe {
            if let Some(window) = self.get_focused_window() {
                windows.push(window);
            }
        }

        Ok(Event {
            windows,
            rule: None,
            keyboard: 0,
            mouse: 0,
            screenshots: None,
            network: None,
            seconds_since_last_input: user_idle::UserIdle::get_time()
                .map(|e| e.duration())
                .map_err(|e| anyhow::Error::msg(e))
                .context("Couldn't get duration since user input")
                .unwrap_or_else(|e| {
                    log::warn!("{}", e);
                    Duration::ZERO
                })
                .as_secs(),
        })
    }
}

/// Checks the Accessibility permission, if not available prompts the user for it.
unsafe fn check_accessibility_permission() -> bool {
    let mut dict: CFMutableDictionary<CFString, CFNumber> = CFMutableDictionary::new();

    dict.add(
        &CFString::from_void(kAXTrustedCheckOptionPrompt as *const c_void).to_owned(),
        &1i64.into(),
    );

    let app_has_permissions =
        AXIsProcessTrustedWithOptions(dict.into_untyped().to_void() as *const _);

    app_has_permissions
}

pub unsafe fn update_frontmost_application_pid() {
    run_loop();

    let pid = get_frontmost_application_pid();

    FRONTMOST_APPLICATION_PID.store(pid, Ordering::Relaxed);
}

unsafe fn get_frontmost_application_pid() -> i32 {
    let shared_workspace: *mut Object = msg_send![class!(NSWorkspace), sharedWorkspace];

    let frontmost_applicaiton: *mut Object = msg_send![shared_workspace, frontmostApplication];

    let pid: i32 = msg_send![frontmost_applicaiton, processIdentifier];

    pid
}

/// Run this function before trying to access shared data outside of this process's context,
/// so that you synchronize and get access to the latest available data.
unsafe fn run_loop() {
    debug!("Running Run Loop");

    let run_loop: *mut Object = msg_send![class!(NSRunLoop), mainRunLoop];

    let date: *mut Object = msg_send![class!(NSDate), dateWithTimeIntervalSinceNow:0];

    let _: () = msg_send![run_loop, runUntilDate: date];
}

/// Frees any Objects
pub unsafe fn release(object: *mut Object) {
    let _: () = msg_send![object, release];
}

/// Turns an
/// [NSString](https://developer.apple.com/documentation/foundation/nsstring?language=objc) Object into a `&str`.
pub unsafe fn ns_string_to_string(ns_string: *mut Object) -> Option<String> {
    // Get length of name
    let string_size: usize = msg_send![ns_string, lengthOfBytesUsingEncoding: 4];

    // Allocate length of name + 1 (for null terminator)
    let char_ptr = libc::malloc(string_size + 1);

    // Copy the string into the allocated memory
    // encoding: 4 is for specifying that the string has UTF-8 encoding
    let res: bool = msg_send![ns_string, getCString:char_ptr maxLength:string_size + 1 encoding:4];

    release(ns_string);

    if !res {
        libc::free(char_ptr);
        return None;
    }

    let string = CStr::from_ptr(char_ptr as *const i8)
        .to_str()
        .unwrap()
        .to_owned();

    libc::free(char_ptr);

    Some(string)
}
