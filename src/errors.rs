use nix;

error_chain! {
    types {}

    links {}

    foreign_links {
        NixError(nix::Error) #[doc="nix error"];
    }

    errors {
        SupervisorInitError {
            description("Could not initialize supervisor")
            display("Could not initialize supervisor")
        }
    }
}
