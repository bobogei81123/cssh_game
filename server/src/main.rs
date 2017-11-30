#![feature(plugin, custom_attribute)]
#![feature(conservative_impl_trait)]
#![feature(fnbox)]
#![feature(catch_expr)]
#![feature(box_syntax)]
#![feature(vec_remove_item)]
#![cfg_attr(not(feature = "clippy"), allow(unknown_lints))]
#![allow(clone_on_ref_ptr)]

pub extern crate futures;
extern crate futures_cpupool;
extern crate itertools;
extern crate rand;
extern crate tokio_core;
extern crate tokio_timer;

#[macro_use] extern crate serde_derive;

#[macro_use] extern crate slog;

extern crate toml;

use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

#[macro_use] mod macro_utils;
mod common;
mod ws;
mod game;
mod logger;

extern crate iron;
extern crate mount;
extern crate staticfile;

/*
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
*/

fn main() {
    thread::spawn(|| {
        //rocket::ignite()
            //.mount("/", routes![hello, static_file, pimg_file])
            //.launch()
        let mut mount = mount::Mount::new();
        mount.mount("/", staticfile::Static::new(Path::new("../client/"))
                    .cache(Duration::from_secs(60*60*24)));
        mount.mount("/pimg/", staticfile::Static::new(Path::new("../problems/images/")));
        println!("Iron running at 0.0.0.0:8000");
        iron::Iron::new(mount).http("0.0.0.0:8000").unwrap();
    });
    let server = game::GameServer::new();
    server.start();
}
