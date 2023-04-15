use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, arg_required_else_help(true))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(PartialEq, Debug, Clone, Subcommand)]
pub enum Command {
    /// Rename an Unreal Engine project
    RenameProject {
        /// Path to the project to rename
        #[arg(long)]
        project: PathBuf,
        /// New name for the project
        #[arg(long)]
        new_name: String,
    },
    /// Rename an Unreal Engine project plugin
    RenamePlugin {
        /// Path to the project that the plugin is part of
        #[arg(long)]
        project: PathBuf,
        /// Plugin in the project to rename
        #[arg(long)]
        plugin: String,
        /// New name for the plugin
        #[arg(long)]
        new_name: String,
    },
    /// Rename an Unreal Engine project target
    RenameTarget {
        /// Path to the project that the target is part of
        #[arg(long)]
        project: PathBuf,
        /// Target in the project to rename
        #[arg(long)]
        target: String,
        /// New name for the target
        #[arg(long)]
        new_name: String,
    },
    /// Rename an Unreal Engine project module
    RenameModule {
        /// Path to the project that the module is part of
        #[arg(long)]
        project: PathBuf,
        /// Module in the project to rename
        #[arg(long)]
        module: String,
        /// New name for the module
        #[arg(long)]
        new_name: String,
    },
    /// Start an interactive session
    Wizard,
}
