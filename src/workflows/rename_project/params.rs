use std::path::PathBuf;

/// Params needed to rename an Unreal Engine project.
pub struct Params {
    /// The root of the project.
    pub project_root: PathBuf,
    /// The target name for the project.
    pub new_name: String,
}
