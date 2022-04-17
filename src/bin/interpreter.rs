use std::{
    cell::UnsafeCell,
    io::{stdin, Read},
    time::Instant,
    rc::Rc,
};
use rustc_hash::FxHashMap;
use timetrackrs::scripting::*;

fn main() -> anyhow::Result<()> {
    let string = include_str!("script");
    // variable_map, must always outlive executables, if it's dropped any earlier it'll cause
    // undefined behaviour, because only a raw pointer is passed to the executables
    let mut variable_map = FxHashMap::default();
    let mut executables = parse(string, (&mut variable_map as *mut _))?;
    let now = Instant::now();
    for (i, mut executable) in executables.1.iter_mut().enumerate() {
       if let Err(err) = executable.execute() {
            println!("{}", err);
       }
    }

    Ok(())
}
