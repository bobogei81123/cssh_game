pub use slog::{Drain, Logger};

extern crate slog_async;
//extern crate slog_envlogger;
extern crate slog_term;

pub fn make_logger() -> Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    //let drain = slog_envlogger::new(drain);
    let drain = slog_async::Async::new(drain).build().fuse();

    Logger::root(drain.fuse(), o!("who" => "Main"))
}
