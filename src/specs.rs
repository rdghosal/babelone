//! Models encapsulating Python package build specifications.
use serde::{Serialize, Deserialize};

/// Denotes a Python package dependency and its required version,
///
/// # Examples
/// `"pydantic==2.x"`, `"flask<3.0"`
pub type Requirement = String;

/// Encapsulates build requirements defined in a requirements.txt (or similar file).
pub struct Requirements {
    pub requires: Vec<Requirement>,
}

/// Encapsulates build specifications defined in a setup.py file.
pub struct Setup {
    pub package_name: String,
    pub version: Option<String>,
    pub dev_requires: Option<Vec<Requirement>>,
    pub install_requires: Option<Vec<Requirement>>,
    pub setup_requires: Option<Vec<Requirement>>,
}

/// Encapsulates build specifications defined in a pyproject.toml file.
#[derive(Serialize, Deserialize)]
pub struct PyProject {
    #[serde(rename = "build-system")]
    pub build_system: Option<BuildSystem>,
    pub project: Option<Project>,
}

#[derive(Serialize, Deserialize)]
pub struct BuildSystem {
    #[serde(rename = "build-backend")]
    pub build_backend: Option<String>,
    pub requires: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub dependencies: Option<Vec<String>>,
}
