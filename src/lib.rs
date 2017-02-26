#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
extern crate libc;
extern crate nix;

use libc::{waitpid, sigwait, pthread_sigmask, sigemptyset, sigfillset, sigaddset, sigset_t};
#[cfg(not(target_os="linux"))]
use libc::{c_int, sigaction};
use libc::{SIG_BLOCK, SIGCHLD};
use nix::unistd::{fork,ForkResult};

use errors::*;

pub mod errors;

/// Runs the supervisor for the given child tasks
pub fn run_supervisor<F>(child_fun: F) -> Result<()>
    where F: FnOnce() -> Result<()>
{
    init_supervisor().chain_err(|| ErrorKind::SupervisorInitError)?;

    match fork() {
        Err(nix::Error::Sys(errno)) => Err(ErrorKind::ForkError(errno).into()),
        Ok(ForkResult::Child) => child_fun(),
        Ok(ForkResult::Parent { child }) => supervise(child),
        _ => bail!("This should not happen!"),
    }
}

fn init_supervisor() -> Result<()> {
    #[cfg(not(target_os="linux"))]
    unsafe {
        let mut act: sigaction = std::mem::uninitialized();
        if sigemptyset(&mut act.sa_mask) == -1 {
            return Err(ErrorKind::SignalAPIError("sigemptyset".to_owned(), -1).into());
        }
        act.sa_sigaction = std::mem::transmute(&null_handler);
        act.sa_flags = 0;
        if sigaction(SIGCHLD, &act, 0 as *mut sigaction) == -1 {
            return Err(ErrorKind::SignalAPIError("sigaction".to_owned(), -1).into());
        }
    }

    mask_all_signals().chain_err(|| "Could not mask signals")?;

    Ok(())
}

fn supervise(child_pid: i32) -> Result<()> {
    println!("Hey, {}. What's happening?", child_pid);

    let mut signop: i32 = 0;
    let wait_ret = unsafe {
        let mut sigchld_set: sigset_t = std::mem::uninitialized();
        if sigemptyset(&mut sigchld_set) == -1 {
            return Err(ErrorKind::SignalAPIError("sigemptyset".to_owned(), -1).into());
        }
        if sigaddset(&mut sigchld_set, SIGCHLD) == -1 {
            return Err(ErrorKind::SignalAPIError("sigaddset".to_owned(), -1).into());
        }
        sigwait(&sigchld_set, &mut signop)
    };

    if wait_ret != 0 {
        return Err(ErrorKind::SignalAPIError("sigwait".to_owned(), wait_ret).into());
    }

    println!("Waitret: {}, Signop: {}", wait_ret, signop);

    // Wait for a child to finish.
    let mut stat_val: i32 = 0;
    let ret = unsafe { waitpid(child_pid, &mut stat_val, 0) };
    println!("{} returned.", ret);
    util::print_exit_status(stat_val);

    Ok(())
}

fn mask_all_signals() -> Result<()> {
    unsafe {
        let mut signal_mask: sigset_t = std::mem::uninitialized();
        if sigfillset(&mut signal_mask) == -1 {
            return Err(ErrorKind::SignalAPIError("sigfillset".to_owned(), -1).into());
        }
        let ret = pthread_sigmask(SIG_BLOCK, &signal_mask, 0 as *mut sigset_t);
        if ret != 0 {
            return Err(ErrorKind::SignalAPIError("pthread_sigmask".to_owned(), ret).into());
        }
        Ok(())
    }
}

#[cfg(not(target_os="linux"))]
extern "C" fn null_handler(_: c_int) -> () {}

mod util;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
