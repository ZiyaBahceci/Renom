mod changeset;

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use inquire::{validator::Validation, CustomUserError, Select, Text};
use regex::Regex;
use walkdir::WalkDir;

use crate::{engine::Engine, presentation::log, unreal::Plugin};

use self::changeset::generate_changeset;

/// Params needed to rename an Unreal Engine plugin.
pub struct Params {
    /// The root of the project.
    pub project_root: PathBuf,
    /// The specific plugin to rename.
    pub plugin: String,
    /// The target name for the plugin.
    pub new_name: String,
}

/// Context needed to rename an Unreal Engine plugin.
pub struct Context {
    /// The root of the project.
    pub project_root: PathBuf,
    /// The name of the project.
    pub project_name: String,
    /// Plugins for the project.
    pub project_plugins: Vec<Plugin>,
    /// The specific plugin to rename.
    pub target_plugin: Plugin,
    /// The target name for the plugin.
    pub target_name: String,
}

/// Rename an Unreal Engine plugin interactively, soliciting input parameters
/// from the user with validation and guided selection.
pub fn rename_plugin_interactive() -> Result<(), String> {
    let params = get_params_from_user()?;
    rename_plugin(params)
}

/// Rename an Unreal Engine plugin.
pub fn rename_plugin(params: Params) -> Result<(), String> {
    validate_params(&params)?;
    let context = gather_context(&params)?;
    let changeset = generate_changeset(&context);
    let backup_dir = create_backup_dir(&context.project_root)?;
    let mut engine = Engine::new();
    if let Err(e) = engine.execute(changeset, backup_dir) {
        log::error(&e);
        engine.revert()?;
        print_failure_message(&context);
        return Ok(());
    }

    print_success_message(&context);
    Ok(())
}

fn get_params_from_user() -> Result<Params, String> {
    let project_root = get_project_root_from_user()?;
    let project_plugins = detect_project_plugins(&project_root)?;
    let target_plugin = get_target_plugin_from_user(&project_plugins)?;
    let target_name = get_target_name_from_user(&project_plugins)?;

    Ok(Params {
        project_root,
        plugin: target_plugin.name,
        new_name: target_name,
    })
}

fn validate_params(_params: &Params) -> Result<(), String> {
    // @todo
    Ok(())
}

fn gather_context(params: &Params) -> Result<Context, String> {
    let project_name = detect_project_name(&params.project_root)?;
    let project_plugins = detect_project_plugins(&params.project_root)?;
    let target_plugin = project_plugins
        .iter()
        .find(|plugin| plugin.name == params.plugin)
        .unwrap()
        .clone();

    Ok(Context {
        project_root: params.project_root.clone(),
        project_name,
        project_plugins,
        target_plugin,
        target_name: params.new_name.clone(),
    })
}

fn get_project_root_from_user() -> Result<PathBuf, String> {
    Text::new("Project root directory path:")
        .with_validator(validate_project_root_is_dir)
        .with_validator(validate_project_root_contains_project_descriptor)
        .with_validator(validate_project_root_contains_source_dir)
        .prompt()
        .map(|project_root| PathBuf::from(project_root))
        .map_err(|err| err.to_string())
}

fn validate_project_root_is_dir(project_root: &str) -> Result<Validation, CustomUserError> {
    match PathBuf::from(project_root).is_dir() {
        true => Ok(Validation::Valid),
        false => {
            let error_message = "Provided path is not a directory";
            Ok(Validation::Invalid(error_message.into()))
        }
    }
}

fn validate_project_root_contains_project_descriptor(
    project_root: &str,
) -> Result<Validation, CustomUserError> {
    match fs::read_dir(project_root)?
        .filter_map(Result::ok)
        .filter_map(|entry| entry.path().extension().map(OsStr::to_owned))
        .any(|ext| ext == "uproject")
    {
        true => Ok(Validation::Valid),
        false => {
            let error_message = "Provided directory does not contain a .uproject file";
            Ok(Validation::Invalid(error_message.into()))
        }
    }
}

fn validate_project_root_contains_source_dir(
    project_root: &str,
) -> Result<Validation, CustomUserError> {
    match PathBuf::from(project_root).join("Source").is_dir() {
        true => Ok(Validation::Valid),
        false => {
            let error_message = "Provided directory does not contain a Source folder";
            Ok(Validation::Invalid(error_message.into()))
        }
    }
}

/// Detect the name of a project given the path to the project root directory.
/// Assumes that the directory exists and that it contains a project descriptor.
/// Returns an error in case of I/O issues.
fn detect_project_name(project_root: &PathBuf) -> Result<String, String> {
    assert!(project_root.is_dir());

    let project_descriptor = fs::read_dir(project_root)
        .map_err(|err| err.to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().map_or(false, |ext| ext == "uproject"))
        .next()
        .expect("project descriptor should exist");

    project_descriptor
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|name| name.to_owned())
        .ok_or("project name is not valid Unicode".into())
}

/// Detect all plugins in a project given the path to the project root
/// directory. Detects top-level plugins and nested plugins. Returns an error in
/// case of I/O issues.
fn detect_project_plugins(project_root: &PathBuf) -> Result<Vec<Plugin>, String> {
    let plugins_dir = project_root.join("Plugins");
    Ok(WalkDir::new(plugins_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map_or(false, |ext| ext == "uplugin")
        })
        .map(|entry| Plugin {
            root: entry.path().parent().unwrap().to_owned(),
            name: entry
                .path()
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned(),
        })
        .collect())
}

fn get_target_plugin_from_user(plugins: &[Plugin]) -> Result<Plugin, String> {
    Select::new("Choose a plugin:", plugins.to_vec())
        .prompt()
        .map_err(|err| err.to_string())
}

fn get_target_name_from_user(plugins: &[Plugin]) -> Result<String, String> {
    let plugins = plugins.to_vec();
    Text::new("Provide a new name for the plugin:")
        .with_validator(validate_target_name_is_not_empty)
        .with_validator(validate_target_name_is_concise)
        .with_validator(move |input: &str| validate_target_name_is_unique(input, &plugins))
        .with_validator(validate_target_name_is_valid_identifier)
        .prompt()
        .map_err(|err| err.to_string())
}

fn validate_target_name_is_not_empty(target_name: &str) -> Result<Validation, CustomUserError> {
    match !target_name.trim().is_empty() {
        true => Ok(Validation::Valid),
        false => {
            let error_message = "Target name must not be empty";
            Ok(Validation::Invalid(error_message.into()))
        }
    }
}

fn validate_target_name_is_concise(target_name: &str) -> Result<Validation, CustomUserError> {
    let target_name_max_len = 30;
    match target_name.len() <= target_name_max_len {
        true => Ok(Validation::Valid),
        false => {
            let error_message = format!(
                "Target name must not be longer than {} characters",
                target_name_max_len
            );
            Ok(Validation::Invalid(error_message.into()))
        }
    }
}

fn validate_target_name_is_unique(
    target_name: &str,
    plugins: &[Plugin],
) -> Result<Validation, CustomUserError> {
    match plugins.iter().all(|plugin| plugin.name != target_name) {
        true => Ok(Validation::Valid),
        false => {
            let error_message = "Target name must not conflict with another plugin";
            Ok(Validation::Invalid(error_message.into()))
        }
    }
}

fn validate_target_name_is_valid_identifier(
    target_name: &str,
) -> Result<Validation, CustomUserError> {
    let identifier_regex = Regex::new("^[_[[:alnum:]]]*$").expect("regex should be valid");
    match identifier_regex.is_match(target_name) {
        true => Ok(Validation::Valid),
        false => {
            let error_message =
                "Target name must be comprised of alphanumeric characters and underscores only";
            Ok(Validation::Invalid(error_message.into()))
        }
    }
}

fn create_backup_dir(project_root: &Path) -> Result<PathBuf, String> {
    let backup_dir = project_root.join(".renom/backup");
    fs::create_dir_all(&backup_dir).map_err(|err| err.to_string())?;
    Ok(backup_dir)
}

fn print_success_message(context: &Context) {
    log::success(format!(
        "Successfully renamed plugin {} to {}.",
        context.target_plugin.name, context.target_name
    ));
}

fn print_failure_message(context: &Context) {
    log::error(format!(
        "Failed to rename plugin {} to {}.",
        context.target_plugin.name, context.target_name
    ));
}
