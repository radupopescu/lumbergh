extern crate libc;
extern crate errno;

use libc::fork;
use errno::errno;

fn run_child() {
    println!("Hello, Lumbergh.");
}

fn run_supervisor(child_pid: i32) {
    println!("Hey, {}. What's happening?", child_pid);

}

fn main() {
    match unsafe { fork() } {
        -1 => {
            println!("Could not fork child process. Error: {}", errno());
        }
        0 => run_child(),
        pid if pid > 0 => run_supervisor(pid),
        _ => panic!("Shouldn't happen. Exiting."),
    }
}