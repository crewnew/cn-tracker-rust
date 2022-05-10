#![cfg(target_os = "linux")]
use super::super::pc_common::{KEYSTROKES, MOUSE_CLICKS};
use notify::{op::Op, raw_watcher, RawEvent, RecursiveMode, Watcher};
use regex::Regex;
use std::{
    fs::{read_to_string, File, OpenOptions},
    io::Read,
    path::Path,
    time::Duration,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::channel,
    },
    mem,
    thread,
};

const EV_KEY: u16 = 1;

const BTN_LEFT: u16 = 0x110;

const BTN_RIGHT: u16 = 0x111;

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct InputEvent {
    tv_sec: isize,  // from timeval struct
    tv_usec: isize, // from timeval struct
    pub type_: u16,
    pub code: u16,
    pub value: i32,
}

static SHOULD_BREAK: AtomicBool = AtomicBool::new(false);

const DEV_INPUT_PATH: &str = "/dev/input";

fn event_listener(path: impl AsRef<Path>) -> std::io::Result<()> {
    let mut buff = [0 as u8; mem::size_of::<InputEvent>()];
    
    let mut file = File::open(path.as_ref())?;
    
    loop {
        file.read(&mut buff)?;
        
        let event: InputEvent = unsafe {mem::transmute(buff)};
        
        if event.type_ != EV_KEY {
            continue;
        }
        
        if event.value == 1{
        match event.code {
            BTN_LEFT | BTN_RIGHT => {
                MOUSE_CLICKS.fetch_add(1, Ordering::Relaxed);
            },
            _ => {
                KEYSTROKES.fetch_add(1, Ordering::Relaxed);
            }
        };
        }

        thread::sleep(Duration::from_millis(30));
    }

    Ok(())
}

pub fn initiate_event_listeners() -> anyhow::Result<()> {
    let devices = read_to_string("/proc/bus/input/devices")?;

    let devices = parse_proc_bus_input_devices(devices)?;

    let (sender, receiver) = channel();

    let mut watcher = raw_watcher(sender)?;

    for device in devices {
        thread::spawn(move || {
            if let Err(err) = event_listener(device) {
                error!("{}", err);
            }
        });
    }
    
    Ok(())
}

/// Returns an event[0-9] by parsing a given string.
fn parse_handlers(string: impl AsRef<str>) -> Option<String> {
    let string = string.as_ref();

    let split = string.split(' ');

    for word in split {
        if word.contains("event") {
            return Some(format!("{}/{}", DEV_INPUT_PATH, word));
        }
    }

    None
}

/// Accepts a string containing information gained from `/proc/bus/input/devices`.
fn parse_proc_bus_input_devices(string: impl AsRef<str>) -> anyhow::Result<Vec<String>> {
    let devices = string.as_ref();

    let mut vec = vec![];

    let lines = devices.split('\n');

    let mut handlers: &str = "";

    for line in lines {
        if let Some(pos) = line.find("Handlers") {
            handlers = &line[pos + "Handlers".len() + 1..];
        } else if let Some(pos) = line.find("EV") {
            let ev_bitmask = &line[pos + "EV".len() + 1..];

            match ev_bitmask {
                "120013" | "17" | "1f" => (),
                _ => continue,
            };

            let event_name = match parse_handlers(handlers) {
                Some(event_name) => event_name,
                None => continue,
            };

            vec.push(event_name);
        }
    }

    if vec.is_empty() {
        anyhow::bail!("Couldn't find any Keyboards or Mices connected to this device.");
    }

    Ok(vec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_proc_info() {
        let string = r#"
I: Bus=0019 Vendor=0000 Product=0001 Version=0000
N: Name="Power Button"
P: Phys=PNP0C0C/button/input0
S: Sysfs=/devices/LNXSYSTM:00/LNXSYBUS:00/PNP0C0C:00/input/input0
U: Uniq=
H: Handlers=kbd event0 
B: PROP=0
B: EV=3
B: KEY=10000000000000 0

I: Bus=0003 Vendor=0627 Product=0001 Version=0001
N: Name="QEMU QEMU USB Tablet"
P: Phys=usb-0000:00:03.0-1/input0
S: Sysfs=/devices/pci0000:00/0000:00:03.0/usb5/5-1/5-1:1.0/0003:0627:0001.0001/input/input1
U: Uniq=28754-0000:00:03.0-1
H: Handlers=mouse0 event1 
B: PROP=0
B: EV=1f
B: KEY=70000 0 0 0 0
B: REL=900
B: ABS=3
B: MSC=10

I: Bus=0003 Vendor=0627 Product=0001 Version=0001
N: Name="QEMU QEMU USB Mouse"
P: Phys=usb-0000:00:03.0-2/input0
S: Sysfs=/devices/pci0000:00/0000:00:03.0/usb5/5-2/5-2:1.0/0003:0627:0001.0002/input/input2
U: Uniq=89126-0000:00:03.0-2
H: Handlers=mouse1 event2 
B: PROP=0
B: EV=17
B: KEY=70000 0 0 0 0
B: REL=903
B: MSC=10

I: Bus=0003 Vendor=0627 Product=0001 Version=0111
N: Name="QEMU QEMU USB Keyboard"
P: Phys=usb-0000:00:03.0-3/input0
S: Sysfs=/devices/pci0000:00/0000:00:03.0/usb5/5-3/5-3:1.0/0003:0627:0001.0003/input/input3
U: Uniq=68284-0000:00:03.0-3
H: Handlers=sysrq kbd event3 leds 
B: PROP=0
B: EV=120013
B: KEY=1000000000007 ff9f207ac14057ff febeffdfffefffff fffffffffffffffe
B: MSC=10
B: LED=1f
"#;

        let devices = parse_proc_bus_input_devices(string).unwrap();

        assert_eq!(
            devices,
            [
                "/dev/input/event1",
                "/dev/input/event2",
                "/dev/input/event3"
            ]
        );
    }
}
