use std::{collections::HashMap, io::Write};

use rbx_xml::EncodeError;

use rbx_dom_weak::{RbxInstanceProperties, RbxTree, RbxValue};

static PLUGIN_TEMPLATE: &'static str = include_str!("plugin_main_template.lua");

pub struct RunInRbxPlugin<'a> {
    pub port: u16,
    pub server_id: &'a str,
    pub lua_script: &'a str,
}

impl<'a> RunInRbxPlugin<'a> {
    pub fn write<W: Write>(&self, output: W) -> Result<(), EncodeError> {
        let tree = self.build_plugin();
        let root_id = tree.get_root_id();

        rbx_xml::to_writer_default(output, &tree, &[root_id])
    }

    fn build_plugin(&self) -> RbxTree {
        let complete_source = PLUGIN_TEMPLATE
            .replace("{{PORT}}", &self.port.to_string())
            .replace("{{SERVER_ID}}", self.server_id);

        let plugin_script = RbxInstanceProperties {
            name: "run-in-roblox-plugin".to_owned(),
            class_name: "Script".to_owned(),
            properties: {
                let mut properties = HashMap::new();

                properties.insert(
                    "Source".to_owned(),
                    RbxValue::String {
                        value: complete_source,
                    },
                );

                properties
            },
        };

        let main_source = format!("return function()\n{}\nend", self.lua_script);

        let injected_main = RbxInstanceProperties {
            name: "Main".to_owned(),
            class_name: "ModuleScript".to_owned(),
            properties: {
                let mut properties = HashMap::new();

                properties.insert("Source".to_owned(), RbxValue::String { value: main_source });

                properties
            },
        };

        let mut tree = RbxTree::new(plugin_script);
        let root_id = tree.get_root_id();
        tree.insert_instance(injected_main, root_id);

        tree
    }
}
