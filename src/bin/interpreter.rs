use std::{
    cell::RefCell,
    io::{stdin, Read},
    time::Instant,
    rc::Rc,
};
use timetrackrs::scripting::*;

fn main() -> anyhow::Result<()> {
    // 0.25ms parse in debug on m1
    // 0.022ms execution in debug on m1
    // 0.08ms parse in release on m1
    // 0.009ms execution in release on m1
    let string = include_str!("script");
    let now = Instant::now();
    let executables = parse(string);
    println!("Parsed in {}ns", now.elapsed().as_nanos());
    let now = Instant::now();
    for executable in executables {
        executable.execute()?;
    }
    println!("Executed in {}ns", now.elapsed().as_nanos());

    Ok(())
}
