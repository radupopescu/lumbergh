extern crate lumbergh;
extern crate nix;

extern crate env_logger;
#[macro_use]
extern crate log;

use std::rc::Rc;
use std::{thread, time};

use lumbergh::errors::*;
use lumbergh::supervisor::{ChildLifetime, ShutdownType, Strategy, Supervisor, SupervisorFlags,
                           ChildSpecs, ProcessType};
use lumbergh::child::FnWorker;

fn make_child_specs(id: u64) -> ChildSpecs {
    ChildSpecs::new(&format!("simple{}", id),
                    Rc::new(FnWorker::new(|| {
                        thread::sleep(time::Duration::from_secs(1));
                        Ok(())
                    })),
                    ChildLifetime::Permanent,
                    ShutdownType::Timeout(1),
                    ProcessType::Worker)
}

fn run() -> Result<()> {
    env_logger::init().chain_err(|| "Could not initialize env_logger")?;
    if let Some(flags) = SupervisorFlags::new(Strategy::OneForOne, 1, 5) {
        let mut child_specs = Vec::new();
        for idx in 0..3 {
            child_specs.push(make_child_specs(idx));
        }
        Supervisor::new(flags, &child_specs).run()?;
    };
    Ok(())
}

fn main() {
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
