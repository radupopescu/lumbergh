use errors::*;
use supervisor::Supervisable;

pub struct FnWorker<F>
    where F: Fn() -> Result<()>
{
    init_fun: Box<F>,
}

impl<F> FnWorker<F>
    where F: Fn() -> Result<()>
{
    pub fn new(fun: F) -> FnWorker<F> {
        FnWorker { init_fun: Box::new(fun) }
    }
}

impl<F> Supervisable for FnWorker<F>
    where F: Fn() -> Result<()>
{
    fn init(&self) -> Result<()> {
        (self.init_fun)()
    }
    fn finalize(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let _ = FnWorker::new(|| Ok(()));
    }
}
