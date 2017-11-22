/// background is a module for our binary crate.
use std::sync::{Arc, Mutex};
use rocksdb::DB;
use coded::config;
use coded::project;

use std::thread;

use std::path::{Path, PathBuf};

pub fn watch(db: Arc<DB>, conf: Arc<Mutex<config::Config>>) {
    use std::time::Duration;
    // We must clone() here?
    let projects = conf.lock()
        .expect("could not unlock conf")
        .clone()
        .project
        .expect("could not unlock projects");
    println!("projects: {:?}", projects);
    loop {
        thread::sleep(Duration::from_secs(3));
        // I was using map() but I'm not "collecting" yet.
        for proj in projects.iter() {
            let mut path = PathBuf::from(proj.dir.as_str());
            if path.exists() {
                // continue...
                match project::guess_type(path.clone()) {
                    project::ProjectType::Go => {
                        project::analyze_go(&mut path, db.clone());
                    }
                    project::ProjectType::Rust => {}
                };
            };
        }
    }
}