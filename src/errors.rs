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
    }
}
