use crate::content::{load_json_registry, ContentError, Registry};
use std::path::Path;
pub fn load_mod_content(path: &Path) -> Result<Registry, ContentError> { load_json_registry(path) }
