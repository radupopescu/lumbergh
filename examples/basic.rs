extern crate lumbergh;
extern crate nix;

extern crate env_logger;
#[macro_use]
extern crate log;

use std::rc::Rc;
use std::{thread, time};

use nix::unistd::getpid;

use lumbergh::supervisor::{Supervisable, WorkerLifetime, ShutdownType, Strategy, Supervisor,
                           SupervisorFlags, ChildSpecs, ProcessType};
use lumbergh::errors::*;

struct SimpleChild {
    c_id: u64,
}

impl Supervisable for SimpleChild {
    fn init(&self) -> Result<()> {
        thread::sleep(time::Duration::from_secs(1));
        info!("{} - Hello, Lumbergh. This is {}", getpid(), self.c_id);
        Ok(())
    }
    fn finalize(&self) -> Result<()> {
        Ok(())
    }
}

fn run() -> Result<()> {
    if let Some(flags) = SupervisorFlags::new(Strategy::OneForOne, 1, 5) {
        let child_specs = [ChildSpecs::new("simple1",
                                           Rc::new(SimpleChild { c_id: 0 }),
                                           WorkerLifetime::Permanent,
                                           ShutdownType::Timeout(1),
                                           ProcessType::Worker),
                           ChildSpecs::new("simple2",
                                           Rc::new(SimpleChild { c_id: 1 }),
                                           WorkerLifetime::Permanent,
                                           ShutdownType::Timeout(1),
                                           ProcessType::Worker)];
        let supervisor = Supervisor::new(flags, &child_specs);
        supervisor.run()?;
    };
    Ok(())
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
