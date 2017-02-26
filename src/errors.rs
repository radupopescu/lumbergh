use errno::Errno;

error_chain! {
    types {}

    links {}

    foreign_links {}

    errors {
        ForkError(error_num: Errno) {
            description("Couldn't fork child process.")
            display("Couldn't fork child process. Errno: {}", error_num)
        }
        SignalAPIError(function_name: String, error_num: i32) {
            description("Signal API function error.")
            display("Signal API function error: {} returned {}", function_name, error_num)
        }
        SupervisorInitError {
            description("Could not initialize supervisor")
            display("Could not initialize supervisor")
        }
    }
}
