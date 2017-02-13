extern crate lumbergh;

use std::{thread, time};

use lumbergh::run_supervisor;

fn run_child(pid: i32) {
    thread::sleep(time::Duration::from_secs(1));
    println!("Hello, Lumbergh. This is {}", pid);
}

fn main() {
    run_supervisor(move || run_child(0));
}
