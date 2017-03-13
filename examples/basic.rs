extern crate lumbergh;

extern crate env_logger;
#[macro_use]
extern crate log;

use std::{thread, time};

use lumbergh::supervisor::Supervisor;
use lumbergh::errors::*;

fn run_child(pid: i32) -> Result<()> {
    thread::sleep(time::Duration::from_secs(1));
    info!("Hello, Lumbergh. This is {}", pid);

    Ok(())
}

fn run() -> Result<()> {
    let supervisor = Supervisor::new();
    supervisor.run(move || run_child(0))
}

fn main() {
    if let Ok(_) = env_logger::init() {
        if let Err(ref e) = run() {
            error!("error: {}", e);

            for e in e.iter().skip(1) {
                error!("caused by: {}", e);
            }

            if let Some(backtrace) = e.backtrace() {
                error!("backtrace: {:?}", backtrace);
            }

            ::std::process::exit(1)
        }
    }
}
