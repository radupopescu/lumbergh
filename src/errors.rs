use nix;
use log;

error_chain! {
    types {}

    links {}

    foreign_links {
        NixError(nix::Error) #[doc="nix error"];
        LogError(log::SetLoggerError) #[doc="log error"];
    }

    errors {
        SupervisorInitError {
            description("Could not initialize supervisor")
            display("Could not initialize supervisor")
        }
        TimeoutError(t: String) {
            description("Timeout reached.")
            display("Timeout reached at {}", t)
        }
    }
}
