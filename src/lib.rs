mod message_receiver;
mod place_runner;
mod plugin;
mod place;
mod core;

use std::{
    fmt,
    path::{Path, PathBuf},
    error::Error,
};

use crate::{
    core::{run_place, run_script, DEFAULT_PORT, DEFAULT_TIMEOUT},
    place_runner::PlaceRunnerOptions,
    message_receiver::RobloxMessage,
};

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
    pub env: RunEnvironment<'a>,
    pub lua_script: &'a str,
    pub port: Option<u16>,
    pub timeout: Option<u16>,
}

pub fn run(opts: RunOptions) -> Result<Vec<RobloxMessage>, Box<dyn Error>> {
    let place_runner_opts = PlaceRunnerOptions {
        lua_script: opts.lua_script,
        port: opts.port.unwrap_or(DEFAULT_PORT),
        timeout: opts.timeout.unwrap_or(DEFAULT_TIMEOUT),
    };

    let message_queue = match opts.env {
        RunEnvironment::EmptyPlace => run_script(place_runner_opts),
        RunEnvironment::PlaceFile(path) => {
            let extension = match path.extension() {
                Some(e) => e.to_str().unwrap(),
                None => return Err(Box::new(BadPathError {
                    path: path.to_path_buf(),
                })),
            };

            run_place(path, extension, place_runner_opts)
        },
    };

    let mut messages = Vec::new();

    while let Some(message) = message_queue.recv()? {
        messages.push(message);
    }

    Ok(messages)
}