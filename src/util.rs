use libc::{WIFCONTINUED, WIFEXITED, WEXITSTATUS, WIFSIGNALED, WTERMSIG, WIFSTOPPED, WSTOPSIG};

pub fn print_exit_status(val: i32) {
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

