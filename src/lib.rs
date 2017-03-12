#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate nix;

#[cfg(not(target_os="linux"))]
use nix::c_int;
#[cfg(not(target_os="linux"))]
use nix::sys::signal::{SaFlags,SigHandler,SigAction,sigaction};
use nix::sys::signal::{SigSet,Signal};
use nix::sys::wait::waitpid;
use nix::unistd::{fork,ForkResult};

use errors::*;

pub mod errors;

/// Runs the supervisor for the given child tasks
pub fn run_supervisor<F>(child_fun: F) -> Result<()>
    where F: FnOnce() -> Result<()>
{
    init_supervisor().chain_err(|| ErrorKind::SupervisorInitError)?;

    match fork().chain_err(|| "Could not fork process.")? {
        ForkResult::Child => child_fun(),
        ForkResult::Parent { child } => supervise(child),
    }
}

fn init_supervisor() -> Result<()> {
    #[cfg(not(target_os="linux"))]
    {
        let mask = SigSet::empty();
        let flags = SaFlags::empty();
        let handler = SigHandler::Handler(null_handler);
        let action = SigAction::new(handler, flags, mask);
        unsafe { sigaction(Signal::SIGCHLD, &action)?; }
    }

    mask_all_signals()?;

    Ok(())
}

fn supervise(child_pid: i32) -> Result<()> {
    info!("Hey, {}. What's happening?", child_pid);

    let mut sigchld = SigSet::empty();
    sigchld.add(Signal::SIGCHLD);
    sigchld.wait()?;

    // Wait for a child to finish.
    let status = waitpid(child_pid, None)?;
    info!("Status: {:?}", status);

    Ok(())
}

fn mask_all_signals() -> Result<()> {
    Ok(SigSet::all().thread_set_mask()?)
}

#[cfg(not(target_os="linux"))]
extern "C" fn null_handler(_: c_int) -> () {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
