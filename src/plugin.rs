use std::{
    collections::HashMap,
    io::{Write},
};

use rbx_xml::{EncodeError};

use rbx_dom_weak::{RbxValue, RbxTree, RbxInstanceProperties};

static PLUGIN_TEMPLATE: &'static str = include_str!("plugin_main_template.lua");

pub struct RunInRbxPlugin<'a> {
    port: u16,
    timeout: u16,
    lua_script: &'a str,
}

impl<'a> RunInRbxPlugin<'a> {
    pub fn new(port: u16, timeout: u16, lua_script: &'a str) -> RunInRbxPlugin<'a> {
        RunInRbxPlugin {
            port,
            timeout,
            lua_script,
        }
    }

    pub fn write<W: Write>(&self, output: W) -> Result<(), EncodeError> {
        let plugin_folder = self.build_plugin();
        let folder_id = plugin_folder.get_root_id();

        rbx_xml::to_writer_default(output, &plugin_folder, &[folder_id])
    }

    fn build_plugin(&self) -> RbxTree {
        let mut plugin_folder = RbxTree::new(RbxInstanceProperties {
            name: String::from("run-in-roblox-plugin"),
            class_name: String::from("Folder"),
            properties: HashMap::new(),
        });

        let complete_source = PLUGIN_TEMPLATE
            .replace("{{PORT}}", &self.port.to_string())
            .replace("{{TIMEOUT}}", &self.timeout.to_string());

        let plugin_script = RbxInstanceProperties {
            name: format!("run-in-roblox-plugin-{}", self.port),
            class_name: String::from("Script"),
            properties: {
                let mut properties = HashMap::new();

                properties.insert(
                    String::from("Source"),
                    RbxValue::String { value: complete_source },
                );

                properties
            },
        };

        let main_source = format!("{}\nreturn nil", self.lua_script);

        let injected_main = RbxInstanceProperties {
            name: String::from("main"),
            class_name: String::from("ModuleScript"),
            properties: {
                let mut properties = HashMap::new();

                properties.insert(
                    String::from("Source"),
                    RbxValue::String { value: main_source },
                );

                properties
            },
        };

        let folder_id = plugin_folder.get_root_id();
        let plugin_script_id = plugin_folder.insert_instance(plugin_script, folder_id);

        plugin_folder.insert_instance(injected_main, plugin_script_id);

        plugin_folder
    }
}

#[cfg(test)]
mod test_plugin {
    use super::*;

    #[test]
    fn run_in_rbx_plugin_creates_correct_plugin_structure() {
        let port = 8080;
        let seconds = 100;
        let main_script = "print('Done')";
        let plugin = RunInRbxPlugin::new(port, seconds, main_script);

        let plugin_folder = plugin.build_plugin();
        let folder_id = plugin_folder.get_root_id();

        assert_eq!(plugin_folder.descendants(folder_id).count(), 2);

        let expect_plugin_source = PLUGIN_TEMPLATE
            .replace("{{PORT}}", &port.to_string())
            .replace("{{TIMEOUT}}", &seconds.to_string());

        let expect_main = format!("{}\nreturn nil", main_script);

        for descendant in plugin_folder.descendants(folder_id) {
            match descendant.class_name.as_ref() {
                "Script" => {
                    match descendant.properties.get("Source") {
                        Some(RbxValue::String { value }) => assert_eq!(value, &expect_plugin_source),
                        _ => panic!("plugin should have a script with source")
                    }
                },
                "ModuleScript" => {
                    match descendant.properties.get("Source") {
                        Some(RbxValue::String { value }) => assert_eq!(value, &expect_main),
                        _ => panic!("plugin should have the given main source")
                    }
                },
                _ => panic!("unexpected descendant")
            }
        }
    }
}