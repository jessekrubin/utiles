use crate::cli::args::{Cli, CommandsArgs};
use crate::cli::commands::unimplemented_cmd_main;
use crate::UtilesResult;
use clap::Parser;
use serde::Serialize;
use tracing::debug;
#[derive(Debug, Serialize)]
struct CommandInfo {
    name: String,
    #[serde(skip)]
    parent: Option<String>,
    about: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    aliases: Option<Vec<String>>,
    hidden: bool,
}

impl CommandInfo {
    pub fn fmt_name_and_aliases(&self) -> String {
        if let Some(aliases) = &self.aliases {
            format!("{} [{}]", self.name, aliases.join(", "))
        } else {
            self.name.clone()
        }
    }
}
fn cmd_info_recursive<'a>(
    cmd: &'a clap::Command,
    path: Option<&'a str>,
    cmd_info: &mut Vec<CommandInfo>,
) {
    let desc = cmd.get_about();
    let aliases: Vec<String> =
        cmd.get_visible_aliases().map(|s| s.to_string()).collect();
    let name = match path {
        Some(path) => format!("{}::{}", path, cmd.get_name()), // name is a String
        None => cmd.get_name().to_string(),
    };
    let cur_cmd_info = CommandInfo {
        name: name.clone(),
        parent: path.map(|s| s.to_string()),
        about: desc.map(|s| s.to_string()),
        aliases: if aliases.is_empty() {
            None
        } else {
            Some(aliases)
        },
        hidden: cmd.is_hide_set(),
    };
    cmd_info.push(cur_cmd_info);
    for sub in cmd.get_subcommands() {
        cmd_info_recursive(sub, Some(&name), cmd_info);
    }
}
fn list_commands(cmd: &clap::Command) -> Vec<CommandInfo> {
    let mut cmd_infos = Vec::new();
    for sub in cmd.get_subcommands() {
        // cmd_info.extend(
        cmd_info_recursive(sub, None, &mut cmd_infos);
    }
    cmd_infos.sort_by(|a, b| a.name.cmp(&b.name));
    cmd_infos
}

pub fn commands_main(cli: &clap::Command, args: &CommandsArgs) -> UtilesResult<()> {
    let cmds_arr = list_commands(&cli);
    let out_str = if args.table {
        cmds_arr
            .iter()
            .map(|cmd| {
                let name_aliases = cmd.fmt_name_and_aliases();
                format!("{}", name_aliases)
            })
            .collect::<Vec<String>>()
            .join("\n")
    } else {
        serde_json::to_string_pretty(&cmds_arr).expect("json serialization error")
    };
    println!("{}", out_str);
    Ok(())
}
