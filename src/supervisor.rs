use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::vec::Vec;


use futures::Future;
use futures_cpupool::CpuPool;
#[cfg(not(target_os="linux"))]
use nix::c_int;
#[cfg(not(target_os="linux"))]
use nix::sys::signal::{SaFlags, SigAction, SigHandler, sigaction};
use nix::sys::signal::{SigSet, Signal};
use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::{ForkResult, fork};
use time::PreciseTime;
use tokio_timer::Timer;

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

#[derive(Clone)]
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
        thread::spawn(|| {
            let mut sigs = SigSet::empty();
            sigs.add(Signal::SIGCHLD);
            sigs.add(Signal::SIGINT);
            if let Ok(sig) = sigs.wait() {
                match sig {
                    Signal::SIGINT => {
                        warn!("SIGINT!!!!!");
                        // Here, send child processes an exit message
                    }
                    _ => {}
                }
            }
        });

        let pool = CpuPool::new_num_cpus();
        let timer = Timer::default();
        let num_kids = self.child_records.len();
        info!("Supervisor waiting for child processes. {} remaining.",
              num_kids);
        let (tx, rx) = mpsc::channel();
        let records = self.child_records.clone();
        let names = self.child_specs.iter().map(|s| s.id.clone()).collect::<Vec<String>>();
        thread::spawn(move || for idx in 0..num_kids {
            let pid = get_pid_for_idx(&records, idx);
            let birth = records[&pid].time_started;
            let child_name = names[records[&pid].spec_index].clone();
            let txc = tx.clone();
            let wait_future = pool.spawn_fn(move || {
                info!("Waiting for {}", pid);
                match waitpid(pid, None) {
                    Ok(WaitStatus::Exited(_, ret)) => {
                        let life = birth.to(PreciseTime::now())
                            .num_seconds();
                        info!("Status: {} exited after {}s with exit code {}.",
                              child_name,
                              life,
                              ret);
                    }
                    _ => {
                        warn!("Unknown action");
                    }
                };
                let res: Result<i32> = Ok(pid);
                res
            });
            let timeout = pool.spawn(timer.sleep(Duration::from_secs(5))
                .then(|_| {
                    info!("Timeout reached!");
                    Ok(-1)
                }));
            let _ = txc.send(timeout.select(wait_future).map(|(res, _)| res));
        });
        for v in rx {
            if let Ok(pid) = v.wait() {
                info!("{} Exited", pid);
                self.child_records.remove(&pid);
            }
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

fn get_pid_for_idx(records: &HashMap<i32, ChildRecord>, idx: usize) -> i32 {
    let pids = records.iter()
        .filter(|&(_, v)| v.spec_index == idx)
        .map(|(k, _)| *k)
        .collect::<Vec<i32>>();
    pids[0]
}

#[cfg(not(target_os="linux"))]
extern "C" fn null_handler(_: c_int) -> () {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
