#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
extern crate errno;
extern crate libc;

use errno::errno;
use libc::{fork, waitpid, sigwait, pthread_sigmask, sigemptyset, sigfillset, sigaddset, sigset_t};
#[cfg(not(target_os="linux"))]
use libc::{c_int, sigaction};
use libc::{SIG_BLOCK, SIGCHLD};

use errors::*;

pub mod errors;

/// Runs the supervisor for the given child tasks
pub fn run_supervisor<F>(child_fun: F) -> Result<()>
    where F: FnOnce() -> Result<()>
{
    init_supervisor();

    match unsafe { fork() as i32 } {
        -1 => {
            Err(ErrorKind::ForkError(errno()).into())
        }
        0 => child_fun(),
        pid if pid > 0 => supervise(pid),
        _ => bail!("Shouldn't happen. Exiting."),
    }
}

fn init_supervisor() {
    #[cfg(not(target_os="linux"))]
    unsafe {
        let mut act: sigaction = std::mem::uninitialized();
        sigemptyset(&mut act.sa_mask);
        act.sa_sigaction = std::mem::transmute(&null_handler);
        act.sa_flags = 0;
        sigaction(SIGCHLD, &act, 0 as *mut sigaction);
    }

    mask_all_signals();
}

fn supervise(child_pid: i32) -> Result<()> {
    println!("Hey, {}. What's happening?", child_pid);

    let mut signop: i32 = 0;
    let wait_ret = unsafe {
        let mut sigchld_set: sigset_t = std::mem::uninitialized();
        sigemptyset(&mut sigchld_set);
        sigaddset(&mut sigchld_set, SIGCHLD);
        sigwait(&sigchld_set, &mut signop)
    };

    println!("Waitret: {}, Signop: {}", wait_ret, signop);

    // Wait for a child to finish.
    let mut stat_val: i32 = 0;
    let ret = unsafe { waitpid(child_pid, &mut stat_val, 0) };
    println!("{} returned.", ret);
    util::print_exit_status(stat_val);

    Ok(())
}

fn mask_all_signals() -> i32 {
    unsafe {
        let mut signal_mask: sigset_t = std::mem::uninitialized();
        sigfillset(&mut signal_mask);
        pthread_sigmask(SIG_BLOCK, &signal_mask, 0 as *mut sigset_t)
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
