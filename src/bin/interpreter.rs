use rustc_hash::FxHashMap;
use std::thread;
use timetrackrs::graphql::get_rules;
use timetrackrs::scripting::*;

fn main() {
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
