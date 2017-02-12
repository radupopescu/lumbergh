extern crate errno;
extern crate libc;

use errno::errno;
use libc::{fork, waitpid, pthread_sigmask, sigwait, sigemptyset, sigfillset, sigaddset, sigset_t,
           sigaction, c_int};
use libc::{WIFCONTINUED, WIFEXITED, WEXITSTATUS, WIFSIGNALED, WTERMSIG, WIFSTOPPED, WSTOPSIG};
use libc::{SIG_BLOCK, SIGCHLD};

use std::{thread, time};

fn print_exit_status(val: i32) {
    unsafe {
        println!("WIFEXITED: {}", WIFEXITED(val));
        println!("WEXITSTATUS: {}", WEXITSTATUS(val));
        println!("WIFSIGNALED: {}", WIFSIGNALED(val));
        println!("WTERMSIG: {}", WTERMSIG(val));
        println!("WIFSTOPPED: {}", WIFSTOPPED(val));
        println!("WSTOPSIG: {}", WSTOPSIG(val));
        println!("WIFCONTINUED: {}", WIFCONTINUED(val));
    }
}

fn run_child(pid: i32) {
    thread::sleep(time::Duration::from_secs(1));
    println!("Hello, Lumbergh. This is {}", pid);
}

fn run_supervisor(child_pid: i32) {
    println!("Hey, {}. What's happening?", child_pid);

    let mut signop: i32 = 0;
    let wait_ret = unsafe {
        let mut sigchld_set: sigset_t = std::mem::uninitialized();
        sigemptyset(&mut sigchld_set);
        sigaddset(&mut sigchld_set, SIGCHLD);
        sigwait(&mut sigchld_set, &mut signop)
    };

    println!("Waitret: {}, Signop: {}", wait_ret, signop);

    // Wait for a child to finish.
    let mut stat_val: i32 = 0;
    let ret = unsafe { waitpid(child_pid, &mut stat_val, 0) };
    println!("{} returned.", ret);
    print_exit_status(stat_val);
}

extern "C" fn chld_handler(_: c_int) -> () {}

fn main() {
    unsafe {
        let mut act: sigaction = std::mem::uninitialized();
        sigemptyset(&mut act.sa_mask);
        act.sa_sigaction = std::mem::transmute(&chld_handler);
        act.sa_flags = 0;
        sigaction(SIGCHLD, &act, 0 as *mut sigaction);
    }
    let mask_ret = unsafe {
        let mut signal_mask: sigset_t = std::mem::uninitialized();
        sigfillset(&mut signal_mask);
        pthread_sigmask(SIG_BLOCK, &signal_mask, 0 as *mut sigset_t)
    };
    println!("Mask all signals: {}", mask_ret);

    let child_idx = 0;
    match unsafe { fork() as i32 } {
        -1 => {
            println!("Could not fork child process. Error: {}", errno());
        }
        0 => run_child(child_idx),
        pid if pid > 0 => run_supervisor(pid),
        _ => panic!("Shouldn't happen. Exiting."),
    }
}
