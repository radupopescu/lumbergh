use std::rc::Rc;
use std::vec::Vec;

#[cfg(not(target_os="linux"))]
use nix::c_int;
#[cfg(not(target_os="linux"))]
use nix::sys::signal::{SaFlags, SigHandler, SigAction, sigaction};
use nix::sys::signal::{SigSet, Signal};
use nix::sys::wait::{wait, WaitStatus};
use nix::unistd::{fork, ForkResult};

use errors::*;

pub enum Strategy {
    OneForOne,
    OneForAll,
    RestForOne,
    SimpleOneForOne,
}

#[derive(Clone)]
pub enum WorkerLifetime {
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

pub trait Supervisable {
    fn init(&self) -> Result<()>;
    fn finalize(&self) -> Result<()>;
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
    worker: Rc<Supervisable>,
    restart: WorkerLifetime,
    shutdown: ShutdownType,
    process_type: ProcessType,
}

impl ChildSpecs {
    pub fn new(id: &str,
               worker: Rc<Supervisable>,
               restart: WorkerLifetime,
               shutdown: ShutdownType,
               process_type: ProcessType)
               -> ChildSpecs {
        ChildSpecs {
            id: id.to_owned(),
            worker: worker,
            restart: restart,
            shutdown: shutdown,
            process_type: process_type,
        }
    }
}

pub struct Supervisor {
    flags: SupervisorFlags,
    child_specs: Vec<ChildSpecs>,
}

impl Supervisor {
    pub fn new(flags: SupervisorFlags, child_specs: &[ChildSpecs]) -> Supervisor {
        Supervisor {
            flags: flags,
            child_specs: child_specs.to_vec(),
        }
    }

    /// Runs the supervisor for the given child tasks
    pub fn run(&self) -> Result<()> {
        info!("Supervisor start");
        self.init().chain_err(|| ErrorKind::SupervisorInitError)?;
        for idx in 0..self.child_specs.len() {
            info!("Supervisor spawning child: {}", self.child_specs[idx].id);
            match fork().chain_err(|| "Could not fork process.")? {
                ForkResult::Child => {
                    self.child_specs[idx].worker.init()?;
                    return Ok(());
                }
                ForkResult::Parent { child: _ } => {}
            }
        }
        self.supervise()?;
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
        self.mask_all_signals()?;
        Ok(())
    }

    fn supervise(&self) -> Result<()> {
        let mut sigchld = SigSet::empty();
        sigchld.add(Signal::SIGCHLD);
        sigchld.wait()?;

        let mut active_kids = self.child_specs.len();
        while active_kids > 0 {
            info!("Supervisor waiting for child processes. {} remaining.",
                  active_kids);
            match wait()? {
                WaitStatus::Exited(pid, ret) => {
                    info!("Status: {} exited with code {}", pid, ret);
                }
                _ => {
                    warn!("Unknown action");
                }
            }
            active_kids -= 1;
        }
        Ok(())
    }

    fn mask_all_signals(&self) -> Result<()> {
        Ok(SigSet::all().thread_set_mask()?)
    }
}

#[cfg(not(target_os="linux"))]
extern "C" fn null_handler(_: c_int) -> () {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
