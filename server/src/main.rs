#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(conservative_impl_trait)]

extern crate rocket;
extern crate websocket;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;

use std::path::{Path, PathBuf};
use std::thread;
use rocket::response::NamedFile;

#[get("/")]
fn hello() -> &'static str {
    "Hello, world"
}

#[get("/<path..>", rank = 5)]
fn static_file(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("../client/").join(path)).ok()
}

mod websocket_logic;

fn main() {
    thread::spawn(websocket_logic::start_websocket);

    rocket::ignite().mount("/", routes![hello, static_file]).launch();
}
