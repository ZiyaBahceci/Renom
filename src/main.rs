use clap::Parser;
use renom::{
    cli::{
        Cli,
        Command::{RenameModule, RenamePlugin, RenameProject, RenameTarget, Wizard},
    },
    presentation::log,
    wizard::start_interactive_dialogue,
    workflows::{rename_module, rename_plugin, rename_project, rename_target},
};

fn main() {
    let cli = Cli::parse();
    match cli.command {
        None => { /* noop, clap will handle top-level help and version */ }
        Some(command) => match command {
            RenameProject { project, new_name } => {
                if let Err(e) = rename_project(rename_project::Params {
                    project_root: project,
                    new_name,
                }) {
                    log::error(e);
                }
            }
            RenamePlugin {
                project,
                plugin,
                new_name,
            } => {
                if let Err(e) = rename_plugin(rename_plugin::Params {
                    project_root: project,
                    plugin,
                    new_name,
                }) {
                    log::error(e);
                }
            }
            RenameTarget {
                project,
                target,
                new_name,
            } => {
                if let Err(e) = rename_target(rename_target::Params {
                    project_root: project,
                    target,
                    new_name,
                }) {
                    log::error(e);
                }
            }
            RenameModule {
                project,
                module,
                new_name,
            } => {
                if let Err(e) = rename_module(rename_module::Params {
                    project_root: project,
                    module,
                    new_name,
                }) {
                    log::error(e);
                }
            }
            Wizard => start_interactive_dialogue(),
        },
    };
}
