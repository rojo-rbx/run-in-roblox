use std::{
    collections::HashMap,
    fs::{self, File},
    process::{self, Command, Stdio},
    str,
    time::Duration,
    path::{PathBuf, Path}
};

use rbx_dom_weak::{RbxTree, RbxInstanceProperties};
use tempfile::{tempdir, TempDir};
use colored::Colorize;
use roblox_install::RobloxStudio;

use crate::{
    place::{RunInRbxPlace},
    plugin::{RunInRbxPlugin},
    message_receiver::{Message, OutputLevel, RobloxMessage, MessageReceiver, MessageReceiverOptions},
};

/// A wrapper for process::Child that force-kills the process on drop.
struct KillOnDrop(process::Child);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _dont_care = self.0.kill();
    }
}

pub struct PlaceRunnerOptions<'a> {
    pub port: u16,
    pub timeout: u16,
    pub lua_script: &'a str,
}

pub struct PlaceRunner {
    work_dir: TempDir,
    place_file_path: PathBuf,
    plugin_file_path: PathBuf,
    studio_exec_path: PathBuf,
    port: u16,
}

impl PlaceRunner {
    pub fn new<'a>(tree: RbxTree, options: PlaceRunnerOptions) -> PlaceRunner {
        let work_dir = tempdir()
            .expect("Could not create temporary directory");

        let place_file_path = work_dir.path().join("place.rbxlx");

        create_run_in_roblox_place(&place_file_path, tree);

        let studio_install = RobloxStudio::locate()
            .expect("Could not find Roblox Studio installation");

        let plugin_file_path = studio_install.plugins_path()
            .join("run_in_roblox.rbxmx");

        create_run_in_roblox_plugin(&plugin_file_path, options.port, options.timeout, options.lua_script);

        PlaceRunner {
            work_dir,
            place_file_path,
            plugin_file_path,
            studio_exec_path: studio_install.exe_path(),
            port: options.port,
        }
    }

    pub fn run(&self) {
        let message_receiver = MessageReceiver::start(MessageReceiverOptions {
            port: self.port,
        });

        let _studio_process = KillOnDrop(Command::new(&self.studio_exec_path)
            .arg(format!("{}", self.place_file_path.display()))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Couldn't start Roblox Studio"));

        match message_receiver.recv_timeout(Duration::from_secs(10)).expect("Timeout reached") {
            Message::Start => {},
            _ => panic!("Invalid first message received"),
        }

        while {
            let message = message_receiver.recv();

            self.process_messages(message)
        } {}

        message_receiver.stop();
        let _dont_care = fs::remove_file(&self.plugin_file_path);
    }

    fn process_messages(&self, message: Message) -> bool {
        match message {
            Message::Start => true,
            Message::Stop => false,
            Message::Messages(mut roblox_messages) => {
                roblox_messages.drain(..).for_each(|message| {
                    match message {
                        RobloxMessage::Output {level, body} => {
                            println!("{}", match level {
                                OutputLevel::Print => body.normal(),
                                OutputLevel::Info => body.cyan(),
                                OutputLevel::Warning => body.yellow(),
                                OutputLevel::Error => body.red(),
                            });
                        },
                    }
                });
                true
            },
        }
    }
}

pub fn open_rbx_place_file(path: &Path, extension: &str) -> RbxTree {
    let mut file = File::open(path)
        .expect("Couldn't open file");

    let mut tree = RbxTree::new(RbxInstanceProperties {
        name: String::from("Place"),
        class_name: String::from("DataModel"),
        properties: HashMap::new(),
    });
    let root_id = tree.get_root_id();

    match extension {
        "rbxl" => rbx_binary::decode(&mut tree, root_id, &mut file)
            .expect("Couldn't decode binary place file"),
        "rbxlx" => {
            tree = rbx_xml::from_reader_default(file)
                .expect("Couldn't decode XML place file");
        },
        _ => unreachable!(),
    }

    tree
}

fn create_run_in_roblox_place(place_file_path: &PathBuf, tree: RbxTree) {
    let place_file = File::create(&place_file_path)
        .expect("Could not create temporary place file");

    let place_file2 = File::create(Path::new("temp_file.rbxlx"))
        .expect("Could not create temporary place file");

    let place = RunInRbxPlace::new(tree);

    place.write(&place_file).expect("Could not serialize temporary place file to disk");
    place.write(&place_file2).expect("Could not serialize temporary place file to disk");
}

fn create_run_in_roblox_plugin<'a>(plugin_file_path: &PathBuf, port: u16, timeout: u16, lua_script: &'a str) {
    let plugin = RunInRbxPlugin::new(port, timeout, lua_script);

    let plugin_file = File::create(&plugin_file_path)
        .expect("Could not create temporary plugin file");

    plugin.write(plugin_file).expect("Could not serialize plugin file to disk");
}
