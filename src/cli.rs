use std::{collections::HashMap, str::FromStr};

#[derive(PartialEq, Debug, Clone)]
pub enum Command {
    RenameProject,
    RenamePlugin,
    RenameTarget,
    RenameModule,
    Wizard,
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rename-project" => Ok(Command::RenameProject),
            "rename-plugin" => Ok(Command::RenamePlugin),
            "rename-target" => Ok(Command::RenameTarget),
            "rename-module" => Ok(Command::RenameModule),
            "wizard" => Ok(Command::Wizard),
            _ => Err(format!("{s} not recognized as subcommand")),
        }
    }
}

pub fn get_help_text(command: &Option<Command>) -> String {
    let tagline = get_tagline();
    let usage = match &command {
        None => get_base_usage(),
        Some(comm) => match comm {
            Command::RenameProject => get_rename_project_usage(),
            Command::RenamePlugin => get_rename_plugin_usage(),
            Command::RenameTarget => get_rename_target_usage(),
            Command::RenameModule => get_rename_module_usage(),
            Command::Wizard => get_wizard_usage(),
        },
    };
    format!(
        r#"
{tagline}
{usage}
    "#
    )
}

fn get_tagline() -> &'static str {
    "A simple tool to rename Unreal Engine projects"
}

fn get_base_usage() -> String {
    r#"
Usage: renom [command] [options]

Commands:
    rename-project          Rename a project
    rename-plugin           Rename a project plugin
    rename-module           Rename a project module
    rename-target           Rename a project target
    wizard                  Start an interactive session

Options:
    --help                  Print this help page
    --version               Print version
    "#
    .into()
}

fn get_rename_project_usage() -> String {
    r#"
Usage: renom rename-project [options]

Options:
    --project               Path to the project to rename
    --new-name              New name for the project
    "#
    .into()
}

fn get_rename_plugin_usage() -> String {
    r#"
Usage: renom rename-plugin [options]

Options:
    --project               Path to the project that the plugin is part of
    --plugin                Plugin in the project to rename
    --new-name              New name for the plugin
    "#
    .into()
}

fn get_rename_module_usage() -> String {
    r#"
Usage: renom rename-module [options]

Options:
    --project               Path to the project that the module is part of
    --module                Module in the project to rename
    --new-name              New name for the module
    "#
    .into()
}

fn get_rename_target_usage() -> String {
    r#"
Usage: renom rename-target [options]

Options:
    --project               Path to the project that the target is part of
    --target                Target in the project to rename
    --new-name              New name for the target
    "#
    .into()
}

fn get_wizard_usage() -> String {
    r#"
Usage: renom wizard
    "#
    .into()
}

pub fn parse_args(
    args: &[String],
) -> Result<(Option<Command>, HashMap<String, Option<String>>), (Option<Command>, String)> {
    let mut command: Option<Command> = None;
    let mut options = HashMap::new();

    let mut args = args.iter().skip(1); // skip the first arg (program name)
    loop {
        match args.next() {
            None => break,
            Some(arg) => match arg.as_str() {
                comm @ ("rename-project" | "rename-plugin" | "rename-target" | "rename-module"
                | "wizard") => {
                    if command.is_some() {
                        return Err((None, "command cannot be specified more than once".into()));
                    }
                    command = Some(comm.parse().unwrap());
                }
                opt @ ("--project" | "--plugin" | "--module" | "--target" | "--new-name") => {
                    if options.contains_key(opt) {
                        return Err((
                            command,
                            format!("option {opt} cannot be specified more than once"),
                        ));
                    }
                    match args.next() {
                        None => {
                            return Err((command, format!("missing argument for option {opt}")))
                        }
                        Some(opt_arg) => {
                            options.insert(opt.into(), Some(opt_arg.into()));
                            continue;
                        }
                    }
                }
                opt @ ("--help" | "--version") => {
                    options.insert(opt.into(), None);
                }
                _ => return Err((command, format!("unknown argument provided: {arg}"))),
            },
        };
    }

    let command_movable = command.clone();
    match &command {
        None => validate_base_command_options(&options).map_err(|err| (command_movable, err))?,
        Some(comm) => {
            match comm {
                Command::RenameProject => validate_rename_project_options(&options)
                    .map_err(|err| (command_movable, err))?,
                Command::RenamePlugin => validate_rename_plugin_options(&options)
                    .map_err(|err| (command_movable, err))?,
                Command::RenameTarget => validate_rename_target_options(&options)
                    .map_err(|err| (command_movable, err))?,
                Command::RenameModule => validate_rename_module_options(&options)
                    .map_err(|err| (command_movable, err))?,
                Command::Wizard => {
                    validate_wizard_options(&options).map_err(|err| (command_movable, err))?
                }
            }
        }
    }

    Ok((command, options))
}

fn validate_base_command_options(options: &HashMap<String, Option<String>>) -> Result<(), String> {
    if let Some((key, _)) = options
        .iter()
        .find(|(key, _)| !matches!(key.as_str(), "--help" | "--version"))
    {
        return Err(format!("option {key} is not supported for this operation"));
    }
    if !options.contains_key("--help") && !options.contains_key("--version") {
        return Err(format!("--help, --version, or command must be specified"));
    }
    Ok(())
}

fn validate_rename_project_options(
    options: &HashMap<String, Option<String>>,
) -> Result<(), String> {
    if !options.contains_key("--project") {
        return Err("--project must be specified".into());
    }
    if !options.contains_key("--new-name") {
        return Err("--new-name must be specified".into());
    }
    if let Some((key, _)) = options
        .iter()
        .find(|(key, _)| !matches!(key.as_str(), "--project" | "--new-name"))
    {
        return Err(format!("option {key} is not supported for this operation"));
    }
    Ok(())
}

fn validate_rename_plugin_options(options: &HashMap<String, Option<String>>) -> Result<(), String> {
    if !options.contains_key("--project") {
        return Err("--project must be specified".into());
    }
    if !options.contains_key("--plugin") {
        return Err("--plugin must be specified".into());
    }
    if !options.contains_key("--new-name") {
        return Err("--new-name must be specified".into());
    }
    if let Some((key, _)) = options
        .iter()
        .find(|(key, _)| !matches!(key.as_str(), "--project" | "--plugin" | "--new-name"))
    {
        return Err(format!("option {key} is not supported for this operation"));
    }
    Ok(())
}

fn validate_rename_module_options(options: &HashMap<String, Option<String>>) -> Result<(), String> {
    if !options.contains_key("--project") {
        return Err("--project must be specified".into());
    }
    if !options.contains_key("--module") {
        return Err("--module must be specified".into());
    }
    if !options.contains_key("--new-name") {
        return Err("--new-name must be specified".into());
    }
    if let Some((key, _)) = options
        .iter()
        .find(|(key, _)| !matches!(key.as_str(), "--project" | "--module" | "--new-name"))
    {
        return Err(format!("option {key} is not supported for this operation"));
    }
    Ok(())
}

fn validate_rename_target_options(options: &HashMap<String, Option<String>>) -> Result<(), String> {
    if !options.contains_key("--project") {
        return Err("--project must be specified".into());
    }
    if !options.contains_key("--target") {
        return Err("--target must be specified".into());
    }
    if !options.contains_key("--new-name") {
        return Err("--new-name must be specified".into());
    }
    if let Some((key, _)) = options
        .iter()
        .find(|(key, _)| !matches!(key.as_str(), "--project" | "--target" | "--new-name"))
    {
        return Err(format!("option {key} is not supported for this operation"));
    }
    Ok(())
}

fn validate_wizard_options(options: &HashMap<String, Option<String>>) -> Result<(), String> {
    if let Some((key, _)) = options.iter().next() {
        return Err(format!("option {key} is not supported for this operation"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::cli::Command;

    use super::parse_args;

    #[test]
    fn parse_args_should_return_command_and_options_if_args_valid() {
        let args = vec![
            String::from("renom"),
            String::from("rename-project"),
            String::from("--project"),
            String::from("test/Code"),
            String::from("--new-name"),
            String::from("Codex"),
        ];

        let (command, options) = parse_args(&args).unwrap();

        assert!(command.is_some());
        assert_eq!(command.unwrap(), Command::RenameProject);
        assert_eq!(options.len(), 2);
        assert_eq!(options["--project"].as_ref().unwrap().as_str(), "test/Code");
        assert_eq!(options["--new-name"].as_ref().unwrap().as_str(), "Codex");
    }

    #[test]
    fn parse_args_should_return_error_if_unknown_option() {
        let args = vec![
            String::from("renom"),
            String::from("rename-project"),
            String::from("--unknown"),
        ];

        let result = parse_args(&args);

        assert!(result.is_err());
    }

    #[test]
    fn parse_args_should_return_error_if_opt_arg_missing() {
        let args = vec![
            String::from("renom"),
            String::from("rename-project"),
            String::from("--project"),
        ];

        let result = parse_args(&args);

        assert!(result.is_err());
    }

    #[test]
    fn parse_args_should_return_error_if_singular_opt_specified_more_than_once() {
        let args = vec![
            String::from("renom"),
            String::from("rename-project"),
            String::from("--project"),
            String::from("test/Code"),
            String::from("--project"),
            String::from("test/Node"),
        ];

        let result = parse_args(&args);

        assert!(result.is_err());
    }

    #[test]
    fn parse_args_should_return_error_if_more_than_one_subcommand_specified() {
        let args = vec![
            String::from("renom"),
            String::from("rename-project"),
            String::from("rename-plugin"),
        ];

        let result = parse_args(&args);

        assert!(result.is_err());
    }

    #[test]
    fn parse_args_should_return_error_if_unsupported_option_for_subcommand_specified() {
        let args = vec![
            String::from("renom"),
            String::from("rename-project"),
            String::from("--project"),
            String::from("test/Code"),
            String::from("--new-name"),
            String::from("Codex"),
            String::from("--module"),
            String::from("Code"),
        ];

        let result = parse_args(&args);

        assert!(result.is_err());
    }

    #[test]
    fn parse_args_should_return_error_if_missing_option_for_subcommand() {
        let args = vec![
            String::from("renom"),
            String::from("rename-project"),
            String::from("--project"),
            String::from("test/Code"),
        ];

        let result = parse_args(&args);

        assert!(result.is_err());
    }
}
