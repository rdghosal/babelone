//! Models encapsulating Python package build specifications.
use serde::{Deserialize, Serialize};
use pyo3::prelude::*;

#[pyclass]
pub enum PyBuildSpec {
    Requirements,
    Setup,
    PyProject,
}

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
#[derive(Debug)]
pub struct Setup {
    pub package_name: Option<String>,
    pub version: Option<String>,
    pub dev_requires: Option<Vec<Requirement>>,
    pub install_requires: Option<Vec<Requirement>>,
    pub setup_requires: Option<Vec<Requirement>>,
}

/// Encapsulates build specifications defined in a pyproject.toml file.
#[derive(Debug, Serialize, Deserialize)]
pub struct PyProject {
    pub project: Option<Project>,
    #[serde(rename = "build-system")]
    pub build_system: Option<BuildSystem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildSystem {
    #[serde(rename = "build-backend")]
    pub build_backend: Option<String>,
    pub requires: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: Option<String>,
    pub version: Option<String>,
    pub dependencies: Option<Vec<String>>,
}

impl PyProject {
    pub fn from_setup(setup: Setup) -> Self {
        let name = setup.package_name;
        let version = setup.version;
        let dependencies = setup.install_requires;
        let requires = setup.setup_requires;
        let build_system = if requires.is_some() {
            Some(BuildSystem {
                requires,
                build_backend: None, // TODO
            })
        } else {
            None
        };
        let project = Some(Project {
            name,
            version,
            dependencies,
        });
        Self {
            project,
            build_system,
        }
    }
}
