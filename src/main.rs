mod message_receiver;
mod place_runner;
mod plugin;

use std::{path::PathBuf, process, sync::mpsc, thread};

use anyhow::anyhow;
use colored::Colorize;
use fs_err as fs;
use structopt::StructOpt;
use tempfile::tempdir;

use crate::{
    message_receiver::{OutputLevel, RobloxMessage},
    place_runner::PlaceRunner,
};

#[derive(Debug, StructOpt)]
struct Options {
    /// A path to the place file to open in Roblox Studio. If not specified, an
    /// empty place file is used.
    #[structopt(long("place"))]
    place_path: Option<PathBuf>,

    /// The universe of the place id to open in Roblox Studio. If not specified,
    /// the place-id will not be used
    #[structopt(long("universe"))]
    universe_id: Option<u64>,

    /// The place id to open in Roblox Studio. If not specified, the place-id
    /// will not be used
    #[structopt(long("place-id"))]
    place_id: Option<u64>,

    /// A path to the script to run in Roblox Studio.
    ///
    /// The script will be run at plugin-level security.
    #[structopt(long("script"))]
    script_path: PathBuf,
}

fn run(options: Options) -> Result<i32, anyhow::Error> {
    // Additional parameters that we may want to pass to roblox Studio
    let mut params = vec![];

    // Create a temp directory to house our place, even if a path is given from
    // the command line. This helps ensure Studio won't hang trying to tell the
    // user that the place is read-only because of a .lock file.
    let temp_place_folder = tempdir()?;

    match &options.place_path {
        Some(place_path) => {
            let extension = place_path
                .extension()
                .ok_or_else(|| anyhow!("Place file did not have a file extension"))?
                .to_str()
                .ok_or_else(|| anyhow!("Place file extension had invalid Unicode"))?;

            let temp_place_path = temp_place_folder
                .path()
                .join(format!("run-in-roblox-place.{}", extension));

            fs::copy(place_path, &temp_place_path)?;

            params.push(format!("{}", temp_place_path.display()));
        }
        None => {
            // Fall back to testing universe and place-id
            // If we have both a universe and place id, we need to pass them to Studio
            match (&options.universe_id, &options.place_id) {
                (Some(universe_id), Some(place_id)) => {
                    params.extend_from_slice(&vec![
                        // This tells studio to open the place
                        "-task".to_string(),
                        "EditPlace".to_string(),
                        // The universe to open
                        "-universeId".to_string(),
                        universe_id.to_string(),
                        // The place to open
                        "-placeId".to_string(),
                        place_id.to_string(),
                    ]);
                }
                _ => {
                    unimplemented!("run-in-roblox must have a place path or universe and place id");
                }
            };
        }
    }



    let script_contents = fs::read_to_string(&options.script_path)?;

    // Generate a random, unique ID for this session. The plugin we inject will
    // compare this value with the one reported by the server and abort if they
    // don't match.
    let server_id = format!("run-in-roblox-{:x}", rand::random::<u128>());

    let place_runner = PlaceRunner {
        port: 50312,
        server_id: server_id.clone(),
        lua_script: script_contents.clone(),
        args: params,
    };

    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        place_runner.run(sender).unwrap();
    });

    let mut exit_code = 0;

    while let Some(message) = receiver.recv()? {
        match message {
            RobloxMessage::Output { level, body } => {
                let colored_body = match level {
                    OutputLevel::Print => body.normal(),
                    OutputLevel::Info => body.cyan(),
                    OutputLevel::Warning => body.yellow(),
                    OutputLevel::Error => body.red(),
                };

                println!("{}", colored_body);

                if level == OutputLevel::Error {
                    exit_code = 1;
                }
            }
        }
    }

    Ok(exit_code)
}

fn main() {
    let options = Options::from_args();

    {
        let log_env = env_logger::Env::default().default_filter_or("warn");

        env_logger::Builder::from_env(log_env)
            .format_timestamp(None)
            .init();
    }

    match run(options) {
        Ok(exit_code) => process::exit(exit_code),
        Err(err) => {
            log::error!("{:?}", err);
            process::exit(2);
        }
    }
}
