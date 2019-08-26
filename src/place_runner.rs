use std::{
    collections::HashMap,
    fs::{self, File},
    process::{self, Command, Stdio},
    str,
    time::Duration,
    path::{PathBuf, Path},
    sync::mpsc,
};

use rbx_dom_weak::{RbxTree, RbxInstanceProperties};
use tempfile::{tempdir, TempDir};
use roblox_install::RobloxStudio;

use crate::{
    place::{RunInRbxPlace},
    plugin::{RunInRbxPlugin},
    message_receiver::{Message, RobloxMessage, MessageReceiver, MessageReceiverOptions},
};

/// A wrapper for process::Child that force-kills the process on drop.
struct KillOnDrop(process::Child);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        let _ignored = self.0.kill();
    }
}

pub struct PlaceRunnerOptions<'a> {
    pub port: u16,
    pub timeout: u16,
    pub lua_script: &'a str,
}

pub struct PlaceRunner {
    _work_dir: TempDir,
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

        create_run_in_roblox_place(&place_file_path, tree, options.port);

        let studio_install = RobloxStudio::locate()
            .expect("Could not find Roblox Studio installation");

        let plugin_file_path = studio_install.plugins_path()
            .join(format!("run_in_roblox-{}.rbxmx", options.port));

        create_run_in_roblox_plugin(&plugin_file_path, options.port, options.timeout, options.lua_script);

        PlaceRunner {
            // Tie the lifetime of this TempDir to our own lifetime, so that it
            // doesn't get cleaned up until we're dropped
            _work_dir: work_dir,
            place_file_path,
            plugin_file_path,
            studio_exec_path: studio_install.application_path().to_path_buf(),
            port: options.port,
        }
    }

    pub fn run_with_sender(&self, message_processor: mpsc::Sender<Option<RobloxMessage>>) {
        let message_receiver = MessageReceiver::start(MessageReceiverOptions {
            port: self.port,
        });

        let _studio_process = KillOnDrop(Command::new(&self.studio_exec_path)
            .arg(format!("{}", self.place_file_path.display()))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Couldn't start Roblox Studio"));

        match message_receiver.recv_timeout(Duration::from_secs(20)).expect("Timeout reached") {
            Message::Start => {},
            _ => panic!("Invalid first message received"),
        }

        loop {
            let message = message_receiver.recv();
            match message {
                Message::Start => {},
                Message::Stop => {
                    message_processor.send(None).expect("Could not send stop message");
                    break;
                },
                Message::Messages(roblox_messages) => {
                    for message in roblox_messages.into_iter() {
                        message_processor.send(Some(message)).expect("Could not send message");
                    }
                }
            }
        }

        message_receiver.stop();
        let _ignored = fs::remove_file(&self.plugin_file_path);
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

fn create_run_in_roblox_place(place_file_path: &PathBuf, tree: RbxTree, port: u16) {
    let place_file = File::create(place_file_path)
        .expect("Could not create temporary place file");

    let place = RunInRbxPlace::new(tree, port);

    place.write(&place_file).expect("Could not serialize temporary place file to disk");
}

fn create_run_in_roblox_plugin<'a>(plugin_file_path: &PathBuf, port: u16, timeout: u16, lua_script: &'a str) {
    let plugin = RunInRbxPlugin::new(port, timeout, lua_script);

    let plugin_file = File::create(&plugin_file_path)
        .expect("Could not create temporary plugin file");

    plugin.write(plugin_file).expect("Could not serialize plugin file to disk");
}
