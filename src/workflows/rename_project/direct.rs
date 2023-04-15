use std::fs;

use crate::engine::Engine;

use super::{changeset::generate_changeset, gather_context_from_params, validate_params, Params};

/// Rename an Unreal Engine project.
pub fn rename_project(params: Params) -> Result<(), String> {
    validate_params(&params)?;
    let context = gather_context_from_params(&params)?;
    let changeset = generate_changeset(&context);
    let mut engine = Engine::new();
    let backup_dir = context.project_root.join(".renom").join("backup");
    fs::create_dir_all(&backup_dir).unwrap();
    if let Err(e) = engine.execute(changeset, backup_dir) {
        match engine.revert() {
            Ok(_) => return Err(e),
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
