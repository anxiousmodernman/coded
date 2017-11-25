#![feature(plugin)]
#![plugin(rocket_codegen)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(unused_must_use)]
#![allow(dead_code)]

extern crate bincode;
extern crate rocksdb;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate walkdir;
extern crate rocket;
extern crate chrono;

// we must do our mods in src/lib.rs
// so that we can use cargo's integration testing

pub mod config;
pub mod db;
pub mod project;
