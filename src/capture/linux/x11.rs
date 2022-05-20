// https://bitbucket.org/nomeata/arbtt/src/master/src/Capture/X11.hs
// https://docs.rs/x11rb/0.3.0/x11rb/
// Root Window Properties (and Related Messages) https://specifications.freedesktop.org/wm-spec/latest/ar01s03.html

#![allow(non_snake_case)]

use super::{
    super::{
        pc_common::{self, Event, Process, KEYSTROKES, MOUSE_CLICKS},
        Capturer,
    },
    peripherals::initiate_event_listeners,
    types::*,
};
use crate::{graphql::get_network_info, util};
use anyhow::Context;
use serde_json::{json, Value as J};
use std::{
    collections::{BTreeMap, HashMap},
    sync::atomic::Ordering,
    thread,
    time::Duration,
};
use sysinfo::{PidExt, ProcessExt, SystemExt};
use x11rb::{
    connection::{Connection, RequestConnection},
    protocol::xproto::{get_property, intern_atom, Atom, AtomEnum, ConnectionExt, Window},
};

fn get_property32<Conn: ?Sized + RequestConnection>(
    conn: &Conn,
    window: Window,
    property: Atom,
) -> anyhow::Result<Vec<u32>> {
    // TODO: use helper from https://github.com/psychon/x11rb/pull/172/files
    let reply = get_property(
        conn,
        false,
        window,
        property,
        AtomEnum::ANY,
        0,
        std::u32::MAX,
    )?
    .reply()?;
    Ok(reply.value32().unwrap().collect())
}
fn get_property_text<Conn: ?Sized + RequestConnection>(
    conn: &Conn,
    window: Window,
    property: Atom,
) -> anyhow::Result<String> {
    let reply = get_property(
        conn,
        false,
        window,
        property,
        AtomEnum::ANY,
        0,
        std::u32::MAX,
    )?
    .reply()?;

    Ok(String::from_utf8(reply.value).unwrap())
}
fn single<T: Copy>(v: &[T]) -> T {
    if v.len() != 1 {
        panic!("not one response!!");
    }
    v[0]
}

pub struct X11Capturer<C: Connection> {
    conn: C,
    root_window: u32,
    atom_name_map: HashMap<u32, anyhow::Result<String>>,
}

impl<C: Connection> X11Capturer<C> {
    fn atom(&self, e: &str) -> anyhow::Result<u32> {
        Ok(intern_atom(&self.conn, true, e.as_bytes())?.reply()?.atom)
    }
    fn atom_name(&mut self, e: u32) -> anyhow::Result<String> {
        let conn = &self.conn;
        let z = self
            .atom_name_map
            .entry(e)
            .or_insert_with(|| -> anyhow::Result<String> {
                Ok(String::from_utf8(conn.get_atom_name(e)?.reply()?.name)?)
            });
        match z {
            Err(_e) => Err(anyhow::anyhow!("idk: {}", _e)),
            Ok(ok) => Ok(ok.clone()),
        }
    }
}

pub fn init() -> anyhow::Result<X11Capturer<impl Connection>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    let root_window = screen.root;
    if let Err(err) = initiate_event_listeners() {
        error!("{}", err);
    }
    Ok(X11Capturer {
        conn,
        root_window,
        atom_name_map: HashMap::new(),
    })
}

impl<C: Connection + Send> Capturer for X11Capturer<C> {
    fn capture(&mut self) -> anyhow::Result<Event> {
        let mut system = sysinfo::System::new();
        let NET_CLIENT_LIST = self.atom("_NET_CLIENT_LIST")?;
        let NET_CURRENT_DESKTOP = self.atom("_NET_CURRENT_DESKTOP")?;
        let NET_DESKTOP_NAMES = self.atom("_NET_DESKTOP_NAMES")?;

        let blacklist = [
            self.atom("_NET_WM_ICON")?, // HUUGE
            self.atom("WM_ICON_NAME")?, // invalid unicode _NET_WM_ICON_NAME
            self.atom("WM_NAME")?,      // invalid unicode, use _NET_WM_NAME
        ];

        let current_desktop = single(&get_property32(
            &self.conn,
            self.root_window,
            NET_CURRENT_DESKTOP,
        )?);

        let desktop_names = split_zero(&get_property_text(
            &self.conn,
            self.root_window,
            NET_DESKTOP_NAMES,
        )?);

        let focus = self.conn.get_input_focus()?.reply()?.focus;

        let mut windows = get_property32(&self.conn, self.root_window, NET_CLIENT_LIST)?;

        windows.sort_unstable();

        let mut windows_data = vec![];

        if !windows.contains(&focus) {
            println!("Focussed thing is not in window list!!");
        }

        for window in windows {
            let props = self.conn.list_properties(window)?.reply()?.atoms;
            let mut propmap: BTreeMap<String, J> = BTreeMap::new();
            let mut pid = None;
            for prop in props {
                if blacklist.contains(&prop) {
                    continue;
                }
                let val = get_property(
                    &self.conn,
                    false,
                    window,
                    prop,
                    AtomEnum::ANY,
                    0,
                    std::u32::MAX,
                )?
                .reply()?;
                assert!(val.bytes_after == 0);
                let prop_name = self.atom_name(prop)?;
                let prop_type = self.atom_name(val.type_)?;
                if prop_name == "_NET_WM_PID" && prop_type == "CARDINAL" {
                    pid = val
                        .value32()
                        .map(|e| e.collect::<Vec<_>>())
                        .map(|e| single(&e));
                }
                let pval = match (prop_name.as_str(), prop_type.as_str(), val.format) {
                    (_, "UTF8_STRING", _) | (_, "STRING", _) => {
                        let QQQ = val.value.clone();
                        let s = String::from_utf8(val.value).map_err(|e| {
                            println!("str {} was!! {:x?}", &prop_name, QQQ);
                            e
                        })?;
                        // if(s[s.len() - 1] == '\0') return
                        J::String(s)
                    }
                    (_, "ATOM", _) => {
                        let vec = val.value32().expect("atom value not 32 bit");
                        //let vec = to_u32s(&val.value).into_iter().map(|e| J::Number(e)).collect();
                        json!({
                            "type": format!("{}/{}", prop_type, val.format),
                            "value": vec.into_iter().map(|e| self.atom_name(e)).collect::<anyhow::Result<Vec<_>>>()?
                        })
                    }
                    (_, "CARDINAL", 32) | (_, "WINDOW", _) => {
                        let vec: Vec<_> = val.value32().expect("atom value not 32 bit").collect();
                        //let vec = to_u32s(&val.value).into_iter().map(|e| J::Number(e)).collect();
                        json!({
                            "type": format!("{}/{}", prop_type, val.format),
                            "value": vec
                        })
                    }
                    _ => json!({
                        "type": format!("{}/{}", prop_type, val.format),
                        "value": hex::encode(val.value)
                    }),
                };
                propmap.insert(prop_name, pval);
            }

            let process: Option<Process> = if let Some(pid) = pid {
                system.refresh_process(sysinfo::Pid::from_u32(pid));
                if let Some(procinfo) = system.process(sysinfo::Pid::from_u32(pid as u32)) {
                    Some(procinfo.into())
                } else {
                    println!(
                        "could not get process by pid {} for window {} ({})",
                        pid,
                        window,
                        json!(&propmap)
                    );
                    None
                }
            } else {
                None
            };

            let geo = self.conn.get_geometry(window)?.reply()?;
            let coords = self
                .conn
                .translate_coordinates(window, self.root_window, 0, 0)?
                .reply()?;

            if let Some(process) = process {
                windows_data.push(pc_common::Window {
                    title: propmap.get("_NET_WM_NAME").map(|title| title.to_string()),
                    process,
                });
            }
        }
        let xscreensaver =
            x11rb::protocol::screensaver::query_info(&self.conn, self.root_window)?.reply()?;
        // see XScreenSaverQueryInfo at https://linux.die.net/man/3/xscreensaverunsetattributes

        let data = Event {
            windows: windows_data,
            rule: None,
            keyboard: 0,
            mouse: 0,
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
        };
        Ok(data)
    }
}
