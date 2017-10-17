#![feature(plugin, custom_attribute)]
#![plugin(rocket_codegen)]
#![feature(conservative_impl_trait, type_ascription, fnbox, vec_remove_item)]
#![allow(non_upper_case_globals)]

#[macro_use]
extern crate lazy_static;

extern crate rand;

extern crate itertools;

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

extern crate toml;

use std::path::{Path, PathBuf};
use std::thread;
use rocket::response::NamedFile;

#[macro_use] mod macro_utils;
mod common;
mod event;
mod ws;
mod game;

#[get("/")]
fn hello() -> Option<NamedFile> {
    static_file("index.html".into())
}

#[get("/<path..>", rank = 5)]
fn static_file(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("../client/").join(path)).ok()
}

#[get("/pimg/<path..>", rank = 3)]
fn pimg_file(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("../problems/images/").join(path)).ok()
}

fn main() {

    thread::spawn(|| rocket::ignite().mount("/", routes![hello, static_file, pimg_file]).launch());
    ws::start();
}
