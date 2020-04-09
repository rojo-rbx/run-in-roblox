mod core;
mod message_receiver;
mod place;
mod place_runner;
mod plugin;

use std::{
    path::{Path, PathBuf},
    process,
};

use anyhow::anyhow;
use clap::{App, Arg};
use colored::Colorize;
use fs_err as fs;
use log::error;
use structopt::StructOpt;
use tempfile::tempdir;

use crate::{
    core::{run_model, run_place, run_script, DEFAULT_PORT, DEFAULT_TIMEOUT},
    message_receiver::{OutputLevel, RobloxMessage},
    place_runner::PlaceRunnerOptions,
};

#[derive(Debug, StructOpt)]
struct Options {
    /// A path to the place file to open in Roblox Studio. If not specified, an
    /// empty place file is used.
    #[structopt(long("place"))]
    place_path: Option<PathBuf>,

    /// A path to the script to run in Roblox Studio.
    ///
    /// The script will be run at plugin-level security.
    #[structopt(long("script"))]
    script_path: PathBuf,
}

fn main() -> Result<(), anyhow::Error> {
    let options = Options::from_args();

    {
        let log_env = env_logger::Env::default().default_filter_or("warn");

        env_logger::Builder::from_env(log_env)
            .format_timestamp(None)
            .init();
    }

    // Create a temp directory to house our place, even if a path is given from
    // the command line. This helps ensure Studio won't hang trying to tell the
    // user that the place is read-only because of a .lock file.
    let temp_place_folder = tempdir()?;
    let temp_place_path;

    match &options.place_path {
        Some(place_path) => {
            let extension = place_path
                .extension()
                .ok_or_else(|| anyhow!("Place file did not have a file extension"))?
                .to_str()
                .ok_or_else(|| anyhow!("Place file extension had invalid Unicode"))?;

            temp_place_path = temp_place_folder
                .path()
                .join(format!("run-in-roblox-place.{}", extension));

            fs::copy(place_path, &temp_place_path)?;
        }
        None => {
            unimplemented!("run-in-roblox with no place argument");
        }
    }

    let script_contents = fs::read_to_string(&options.script_path)?;

    // Generate a random, unique ID for this session. The plugin we inject will
    // compare this value with the one reported by the server and abort if they
    // don't match.
    let server_id = format!("run-in-roblox-{:x}", rand::random::<u128>());

    Ok(())

    // let mut exit_code = 0;

    // while let Some(message) = messages.recv().expect("Problem receiving message") {
    //     if let RobloxMessage::Output {
    //         level: OutputLevel::Error,
    //         ..
    //     } = message
    //     {
    //         exit_code = 1;
    //     }

    //     print_message(&message);
    // }

    // process::exit(exit_code);
}

fn print_message(message: &RobloxMessage) {
    match message {
        RobloxMessage::Output { level, body } => {
            println!(
                "{}",
                match level {
                    OutputLevel::Print => body.normal(),
                    OutputLevel::Info => body.cyan(),
                    OutputLevel::Warning => body.yellow(),
                    OutputLevel::Error => body.red(),
                }
            );
        }
    }
}
