mod message_receiver;
mod place_runner;
mod plugin;
mod place;

use std::{
    fmt,
    collections::HashMap,
    path::{Path, PathBuf},
    error::Error,
};

use rbx_dom_weak::{RbxTree, RbxInstanceProperties};

use crate::{
    place_runner::{PlaceRunner, PlaceRunnerOptions, open_rbx_place_file},
};

const DEFAULT_PORT: u16 = 54023;
const DEFAULT_TIMEOUT: u16 = 15;

#[derive(Debug)]
struct BadPathError {
    path: PathBuf,
}

impl Error for BadPathError {}

impl fmt::Display for BadPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid file type: {}", self.path.display())
    }
}

pub enum RunEnvironment<'a> {
    EmptyPlace,
    PlaceFile(&'a Path),
}

pub struct RunOptions<'a> {
    env: RunEnvironment<'a>,
    lua_script: &'a str,
    port: Option<u16>,
    timeout: Option<u16>,
}

pub fn run(opts: RunOptions) -> Result<(), Box<dyn Error>> {
    let place_runner_opts = PlaceRunnerOptions {
        lua_script: opts.lua_script,
        port: opts.port.unwrap_or(DEFAULT_PORT),
        timeout: opts.timeout.unwrap_or(DEFAULT_TIMEOUT),
    };

    let tree = match opts.env {
        RunEnvironment::EmptyPlace => {
            RbxTree::new(RbxInstanceProperties {
                name: String::from("Place"),
                class_name: String::from("DataModel"),
                properties: HashMap::new(),
            })
        },
        RunEnvironment::PlaceFile(path) => {
            let extension = match path.extension() {
                Some(e) => e.to_str().unwrap(),
                None => return Err(Box::new(BadPathError {
                    path: path.to_path_buf(),
                })),
            };

            open_rbx_place_file(path, extension)
        },
    };

    let place = PlaceRunner::new(tree, place_runner_opts);
    place.run();

    Ok(())
}