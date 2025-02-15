#[macro_use]
extern crate log;
use rustc_hash::FxHashMap;
use std::{sync::Arc, thread, time::Duration};
use timetrackrs::{
    capture::capture_peripherals, graphql::get_user_rules, scripting::*, util::get_os_info,
};

fn main() {
    env_logger::init();
    capture_peripherals();
    let rules = get_user_rules().expect("Couldn't get Rules");
    let os_info = get_os_info();

    let os_type = Arc::new(os_info.os_type);
    let version = Arc::new(os_info.version);
    let batteries = os_info.batteries;
    let hostname = Arc::new(os_info.hostname);
    let username = os_info.username.map(|s| Arc::new(s));
    let machine_id = os_info.machine_id.map(|s| Arc::new(s));

    let mut join_handles = vec![];

    for (project_rule_id, rule_body) in rules {
        let os_type = os_type.clone();
        let version = version.clone();
        let batteries = batteries.clone();
        let hostname = hostname.clone();
        let username = username.clone();
        let machine_id = machine_id.clone();

        let handle = thread::spawn(move || {
            // variable_map, must always outlive executables, if it's dropped any earlier it'll cause
            // undefined behaviour, because only a raw pointer is passed to the executables
            let mut variable_map: VariableMapType = FxHashMap::default();
            let (timeout_duration, mut executables) =
                parse(&rule_body, &mut variable_map as *mut _).expect("Couldn't Parse Body");
            variable_map.insert("RULE_ID", project_rule_id.into());
            variable_map.insert("RULE_BODY", rule_body.into());
            variable_map.insert("OS_TYPE", os_type.into());
            variable_map.insert("VERSION", version.into());
            if let Some(batteries) = batteries {
                variable_map.insert("BATTERIES", (batteries as usize).into());
            }
            variable_map.insert("HOSTNAME", hostname.into());
            if let Some(username) = username {
                variable_map.insert("USERNAME", username.into());
            }
            if let Some(machine_id) = machine_id {
                variable_map.insert("MACHINE_ID", machine_id.into());
            }
            loop {
                thread::sleep(timeout_duration);
                for executable in &mut executables {
                    if let Err(err) = executable.execute() {
                        error!("{}", err);
                    }
                }
            }
        });
        join_handles.push(handle);
    }

    let join_thread = thread::spawn(move || {
        for handle in join_handles {
            handle.join().unwrap();
        }
    });

    #[cfg(target_os = "macos")]
    {
        // Loop for updating the frontmostApplication PID
        use timetrackrs::capture::macos::appkit::update_frontmost_application_pid;
        while !join_thread.is_finished() {
            unsafe {
                update_frontmost_application_pid();
            }
            thread::sleep(Duration::from_secs(1));
        }
    }

    join_thread.join().unwrap();
}
