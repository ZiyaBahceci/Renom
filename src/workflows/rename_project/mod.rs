pub mod changeset;
mod context;
mod direct;

pub use direct::*;

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use inquire::{validator::Validation, Confirm, CustomUserError, Text};
use regex::Regex;

use crate::{engine::Engine, presentation::log};

use self::{changeset::generate_changeset, context::Context};

pub struct Params {
    pub project_root: PathBuf,
    pub new_name: String,
}

pub fn validate_params(params: &Params) -> Result<(), String> {
    if !params.project_root.is_dir() {
        return Err("project root must be a directory".into());
    }
    if !fs::read_dir(&params.project_root)
        .map_err(|err| err.to_string())?
        .filter_map(Result::ok)
        .filter_map(|entry| entry.path().extension().map(OsStr::to_owned))
        .any(|ext| ext == "uproject")
    {
        return Err("project root must contain a project descriptor".into());
    }
    let project_name = detect_project_name(&params.project_root)?;
    if project_name == params.new_name {
        return Err("new name must be different than current project name".into());
    }
    Ok(())
}

pub fn gather_context_from_params(params: &Params) -> Result<Context, String> {
    let project_name = detect_project_name(&PathBuf::from(&params.project_root))?;
    Ok(Context {
        project_root: params.project_root.clone(),
        project_name,
        target_name: params.new_name.clone(),
    })
}

pub fn start_rename_project_workflow() -> Result<(), String> {
    let context = gather_context()?;
    let changeset = generate_changeset(&context);
    let backup_dir = create_backup_dir(&context.project_root)?;
    let mut engine = Engine::new();
    if let Err(err) = engine.execute(changeset, &backup_dir) {
        log::error(&err);
        if user_confirms_revert() {
            engine.revert()?;
        }
        print_failure_message(&context);
        return Ok(());
    }

    print_success_message(&context);
    Ok(())
}

fn gather_context() -> Result<Context, String> {
    let project_root = get_project_root_from_user()?;
    let project_name = detect_project_name(&project_root)?;
    let target_name = get_target_name_from_user()?;
    Ok(Context {
        project_root,
        project_name,
        target_name,
    })
}

fn get_project_root_from_user() -> Result<PathBuf, String> {
    Text::new("Project root directory path:")
        .with_validator(validate_project_root_is_dir)
        .with_validator(validate_project_root_contains_project_descriptor)
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

fn get_target_name_from_user() -> Result<String, String> {
    Text::new("Provide a new name for the project:")
        .with_validator(validate_target_name_is_not_empty)
        .with_validator(validate_target_name_is_concise)
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
    let target_name_max_len = 20;
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

/// Create a directory to store backup files in
fn create_backup_dir(project_root: &Path) -> Result<PathBuf, String> {
    let backup_dir = project_root.join(".renom/backup");
    fs::create_dir_all(&backup_dir).map_err(|err| err.to_string())?;
    Ok(backup_dir)
}

/// Request revert desired from the user.
fn user_confirms_revert() -> bool {
    Confirm::new("Looks like something went wrong. Should we revert the changes made so far?")
        .prompt()
        .unwrap_or(false)
}

fn print_success_message(context: &Context) {
    log::success(format!(
        "Successfully renamed project {} to {}.",
        context.project_name, context.target_name
    ));
}

fn print_failure_message(context: &Context) {
    log::error(format!(
        "Failed to rename project {} to {}.",
        context.project_name, context.target_name
    ));
}
