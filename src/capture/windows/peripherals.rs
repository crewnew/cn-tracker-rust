use super::super::pc_common::{KEYSTROKES, MOUSE_CLICKS};
use std::{convert::TryFrom, ptr, sync::atomic::Ordering, thread};
use winapi::{
    ctypes::c_int,
    shared::{
        minwindef::{LPARAM, LRESULT, UINT, WPARAM},
        windef::{HHOOK, HWND},
    },
    um::winuser::{
        CallNextHookEx, DispatchMessageA, GetMessageA, SetWindowsHookExA, TranslateMessage,
        UnhookWindowsHookEx, HC_ACTION, MSG, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN,
        WM_LBUTTONDOWN,
    },
};

pub fn capture_peripherals() {
    unsafe {
        let keyboard_hhook =
            SetWindowsHookExA(WH_KEYBOARD_LL, Some(hook_callback), ptr::null_mut(), 0);

        let mouse_hhook = SetWindowsHookExA(WH_MOUSE_LL, Some(hook_callback), ptr::null_mut(), 0);

        if keyboard_hhook.is_null() || mouse_hhook.is_null() {
            panic!(
                "Couldn't Setup Hooks, Keyboard: {} Mouse: {}",
                keyboard_hhook.is_null(),
                mouse_hhook.is_null()
            );
        }

        message_loop();

        if UnhookWindowsHookEx(keyboard_hhook) == 0 {
            panic!("Windows Unhook non-zero return");
        }
        debug!("Successfully Unhooked Keyboard");
        if UnhookWindowsHookEx(mouse_hhook) == 0 {
            panic!("Windows Unhook non-zero return");
        }
        debug!("Successfully Unhooked Mouse");
    }
}

/// This function handles the Event Loop, which is necessary in order for the hooks to function.
fn message_loop() {
    println!("Message loop for the Hooks initiated.");
    let mut msg = MSG::default();
    unsafe {
        while 0 == GetMessageA(&mut msg, ptr::null_mut(), 0, 0) {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }
        println!("While loop Ended");
    }
}

unsafe extern "system" fn hook_callback(code: c_int, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code == HC_ACTION {
        match UINT::try_from(w_param).unwrap() {
            WM_KEYDOWN => {
                KEYSTROKES.fetch_add(1, Ordering::Relaxed);
            }
            WM_LBUTTONDOWN => {
                MOUSE_CLICKS.fetch_add(1, Ordering::Relaxed);
            }
            _ => (),
        };
    }
    CallNextHookEx(ptr::null_mut(), code, w_param, l_param)
}
