use std::rc::Rc;
use std::vec::Vec;
use std::collections::HashMap;

#[cfg(not(target_os="linux"))]
use nix::c_int;
#[cfg(not(target_os="linux"))]
use nix::sys::signal::{SaFlags, SigAction, SigHandler, sigaction};
use nix::sys::signal::{SigSet, Signal};
use nix::sys::wait::{WaitStatus, wait};
use nix::unistd::{ForkResult, fork};
use time::PreciseTime;

use errors::*;

pub trait Supervisable {
    fn init(&self) -> Result<()>;
    fn finalize(&self) -> Result<()>;
}

pub enum Strategy {
    OneForOne,
    OneForAll,
    RestForOne,
    SimpleOneForOne,
}

#[derive(Clone)]
pub enum ChildLifetime {
    Permanent,
    Temporary,
    Transient,
}

#[derive(Clone)]
pub enum ProcessType {
    Worker,
    Supervisor,
}

#[derive(Clone)]
pub enum ShutdownType {
    BrutalKill,
    Infinity,
    Timeout(u64),
}

pub struct SupervisorFlags {
    strategy: Strategy,
    intensity: u64,
    period: u64,
}

impl SupervisorFlags {
    pub fn new(strategy: Strategy, intensity: u64, period: u64) -> Option<SupervisorFlags> {
        if period > 0 {
            Some(SupervisorFlags {
                strategy: strategy,
                intensity: intensity,
                period: period,
            })
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct ChildSpecs {
    id: String,
    child: Rc<Supervisable>,
    restart: ChildLifetime,
    shutdown: ShutdownType,
    process_type: ProcessType,
}

impl ChildSpecs {
    pub fn new(id: &str,
               child: Rc<Supervisable>,
               restart: ChildLifetime,
               shutdown: ShutdownType,
               process_type: ProcessType)
               -> ChildSpecs {
        ChildSpecs {
            id: id.to_owned(),
            child: child,
            restart: restart,
            shutdown: shutdown,
            process_type: process_type,
        }
    }
}

struct ChildRecord {
    spec_index: usize,
    time_started: PreciseTime,
}

pub struct Supervisor {
    flags: SupervisorFlags,
    child_specs: Vec<ChildSpecs>,
    child_records: HashMap<i32, ChildRecord>,
}

impl Supervisor {
    pub fn new(flags: SupervisorFlags, specs: &[ChildSpecs]) -> Supervisor {
        Supervisor {
            flags: flags,
            child_specs: specs.to_vec(),
            child_records: HashMap::new(),
        }
    }

    /// Runs the supervisor for the given child tasks
    pub fn run(&mut self) -> Result<()> {
        info!("Supervisor start.");
        self.init().chain_err(|| ErrorKind::SupervisorInitError)?;
        for idx in 0..self.child_specs.len() {
            info!("Supervisor spawning child: {}.", self.child_specs[idx].id);
            match fork().chain_err(|| "Could not fork process.")? {
                ForkResult::Child => {
                    self.child_specs[idx].child.init()?;
                    return Ok(());
                }
                ForkResult::Parent { child } => {
                    self.child_records
                        .insert(child,
                                ChildRecord {
                                    spec_index: idx,
                                    time_started: PreciseTime::now(),
                                });
                }
            }
        }
        self.supervise()?;
        Ok(())
    }

    fn supervise(&mut self) -> Result<()> {
        let mut sigs = SigSet::empty();
        sigs.add(Signal::SIGCHLD);
        sigs.add(Signal::SIGINT);

        match sigs.wait()? {
            Signal::SIGINT => {
                warn!("SIGINT!!!!!");
                // Here, send child processes an exit message
            }
            _ => {}
        }

        let mut active_kids = self.child_specs.len();
        while active_kids > 0 {
            info!("Supervisor waiting for child processes. {} remaining.",
                  active_kids);
            match wait()? {
                WaitStatus::Exited(pid, ret) => {
                    let child_name = &self.child_specs[self.child_records[&pid].spec_index].id;
                    let life = self.child_records[&pid].time_started.to(PreciseTime::now()).num_seconds();
                    info!("Status: {} exited after {}s with exit code {}.", child_name, life, ret);
                    self.child_records.remove(&pid);
                }
                _ => {
                    warn!("Unknown action");
                }
            }
            active_kids -= 1;
        }
        Ok(())
    }

    fn init(&self) -> Result<()> {
        #[cfg(not(target_os="linux"))]
        {
            let mask = SigSet::empty();
            let flags = SaFlags::empty();
            let handler = SigHandler::Handler(null_handler);
            let action = SigAction::new(handler, flags, mask);
            unsafe {
                sigaction(Signal::SIGCHLD, &action)?;
            }
        }

        SigSet::all().thread_set_mask()?;
        Ok(())
    }
}

#[cfg(not(target_os="linux"))]
extern "C" fn null_handler(_: c_int) -> () {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
