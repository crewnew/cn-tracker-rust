fn main() {
    env_logger::init();
    #[cfg(target_os = "linux")]
    {
        use std::sync::atomic::Ordering;
        use timetrackrs::capture::{
            linux::peripherals,
            pc_common::{KEYSTROKES, MOUSE_CLICKS}
        };
        println!("Initiating");
        peripherals::initiate_event_listeners().unwrap();
        
        loop {
            println!("Keystrokes: {} Mouse Clicks: {}", KEYSTROKES.load(Ordering::Relaxed), MOUSE_CLICKS.load(Ordering::Relaxed));
        std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }
}
