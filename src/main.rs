extern crate lumbergh;

use std::{thread, time};

use lumbergh::run_supervisor;
use lumbergh::errors::*;

fn run_child(pid: i32) -> Result<()> {
    thread::sleep(time::Duration::from_secs(1));
    println!("Hello, Lumbergh. This is {}", pid);

    Ok(())
}

fn run() -> Result<()> {
    run_supervisor(move || run_child(0))
}

fn main() {
    if let Err(ref e) = run() {
        println!("error: {}", e);

        for e in e.iter().skip(1) {
            println!("caused by: {}", e);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            println!("backtrace: {:?}", backtrace);
        }

        ::std::process::exit(1);
    }
}

