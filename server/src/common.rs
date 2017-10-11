use slog::{Drain, Logger};
use slog_term;
use slog_async;

pub type Id = usize;

lazy_static! {
    pub static ref logger: Logger = {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();

        Logger::root(drain.fuse(), o!())
    };
}

