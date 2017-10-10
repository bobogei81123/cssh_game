#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(conservative_impl_trait, type_ascription)]
#![allow(non_upper_case_globals)]

#[macro_use]
extern crate lazy_static;

extern crate rand;

#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate slog_async;

extern crate chrono;
extern crate byteorder;
extern crate rocket;
extern crate websocket;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;
extern crate tokio_timer;

#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate serde_json;

use std::path::{Path, PathBuf};
use std::thread;
use rocket::response::NamedFile;
use slog::{Drain, Logger};

#[macro_use] mod macro_utils;

mod game;

#[get("/")]
fn hello() -> Option<NamedFile> {
    static_file("index.html".into())
}

#[get("/<path..>", rank = 5)]
fn static_file(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("../client/").join(path)).ok()
}

lazy_static! {
    static ref logger: Logger = {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();

        Logger::root(drain.fuse(), o!())
    };
}

fn main() {

    thread::spawn(|| rocket::ignite().mount("/", routes![hello, static_file]).launch());
    game::init();
}
