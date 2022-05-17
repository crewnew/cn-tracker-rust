use lazy_static::lazy_static;
use rustc_hash::FxHashMap;
use std::thread;
use timetrackrs::{
    capture::capture_peripherals,
    graphql::get_rules,
    scripting::*,
    util::{get_os_info, OsInfo},
};

lazy_static! {
    static ref OS_INFO: OsInfo = get_os_info();
}

fn main() {
    capture_peripherals();
    let rules = get_rules().expect("Couldn't get Rules");
    let mut join_handles = vec![];

    for rule in rules {
        let handle = thread::spawn(move || {
            // variable_map, must always outlive executables, if it's dropped any earlier it'll cause
            // undefined behaviour, because only a raw pointer is passed to the executables
            let mut variable_map: VariableMapType = FxHashMap::default();
            let (timeout_duration, mut executables) =
                parse(&rule.body, &mut variable_map as *mut _).expect("Couldn't Parse Body");
            variable_map.insert("RULE_ID", rule.id.into());
            variable_map.insert("RULE_BODY", rule.body.into());
            variable_map.insert("OS_TYPE", Variable::StaticStr(&OS_INFO.os_type));
            variable_map.insert("VERSION", Variable::StaticStr(&OS_INFO.version));
            if let Some(batteries) = OS_INFO.batteries {
                variable_map.insert("BATTERIES", (batteries as usize).into());
            }
            variable_map.insert("HOSTNAME", Variable::StaticStr(&OS_INFO.hostname));
            if let Some(username) = &OS_INFO.username {
                variable_map.insert("USERNAME", Variable::StaticStr(username));
            }
            if let Some(machine_id) = &OS_INFO.machine_id {
                variable_map.insert("MACHINE_ID", Variable::StaticStr(machine_id));
            }
            loop {
                thread::sleep(timeout_duration);
                for executable in &mut executables {
                    if let Err(err) = executable.execute() {
                        println!("{}", err);
                    }
                }
            }
        });
        join_handles.push(handle);
    }

    for handle in join_handles {
        handle.join().unwrap();
    }
}
