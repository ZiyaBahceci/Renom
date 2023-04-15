use std::path::PathBuf;

use renom::{
    cli::{self, get_help_text, Command},
    presentation::log,
    wizard,
    workflows::{rename_module, rename_plugin, rename_project, rename_target},
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let parsed_args = cli::parse_args(&args);
    if let Err((command, msg)) = parsed_args {
        let help_text = get_help_text(&command);
        println!("{help_text}\nerror: {msg}");
        return;
    }

    let (command, options) = parsed_args.unwrap();
    match command {
        None => {
            if options.contains_key("--help") {
                let help_text = get_help_text(&command);
                println!("{help_text}");
            } else if options.contains_key("--version") {
                let version = env!("CARGO_PKG_VERSION");
                println!("{version}");
            }
        }
        Some(command) => match command {
            Command::RenameProject => {
                if let Err(e) = rename_project(rename_project::Params {
                    project_root: PathBuf::from(options["--project"].as_ref().unwrap()),
                    new_name: options["--new-name"].as_ref().unwrap().clone(),
                }) {
                    log::error(e);
                }
            }
            Command::RenamePlugin => {
                if let Err(e) = rename_plugin(rename_plugin::Params {
                    project_root: PathBuf::from(options["--project"].as_ref().unwrap()),
                    plugin: options["--plugin"].as_ref().unwrap().clone(),
                    new_name: options["--new-name"].as_ref().unwrap().clone(),
                }) {
                    log::error(e);
                }
            }
            Command::RenameTarget => {
                if let Err(e) = rename_target(rename_target::Params {
                    project_root: PathBuf::from(options["--project"].as_ref().unwrap()),
                    target: options["--target"].as_ref().unwrap().clone(),
                    new_name: options["--new-name"].as_ref().unwrap().clone(),
                }) {
                    log::error(e);
                }
            }
            Command::RenameModule => {
                if let Err(e) = rename_module(rename_module::Params {
                    project_root: PathBuf::from(options["--project"].as_ref().unwrap()),
                    module: options["--module"].as_ref().unwrap().clone(),
                    new_name: options["--new-name"].as_ref().unwrap().clone(),
                }) {
                    log::error(e);
                }
            }
            Command::Wizard => wizard::start_interactive_dialogue(),
        },
    }
}
