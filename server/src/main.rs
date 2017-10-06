#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(conservative_impl_trait)]

extern crate rocket;
extern crate websocket;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;

#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate serde_json;

use std::path::{Path, PathBuf};
use std::thread;
use rocket::response::NamedFile;

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


fn main() {
    thread::spawn(game::init);

    rocket::ignite().mount("/", routes![hello, static_file]).launch();
}
