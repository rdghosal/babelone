//! Models encapsulating Python package build specifications.
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

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

impl Requirements {
    pub fn from_setup(setup: Setup) -> Self {
        // TODO: handle extra_requires.
        let mut requires = Vec::<String>::new();
        if let Some(mut install_requires) = setup.install_requires {
            requires.append(&mut install_requires);
        }
        if let Some(mut setup_requires) = setup.setup_requires {
            requires.append(&mut setup_requires);
        }
        Self { requires }
    }

    pub fn from_pyproject(pyproject: PyProject) -> Self {
        let mut requires = Vec::<String>::new();
        if let Some(project) = pyproject.project {
            if let Some(mut dependencies) = project.dependencies {
                requires.append(&mut dependencies);
            }
        }
        if let Some(build_system) = pyproject.build_system {
            if let Some(mut reqs) = build_system.requires {
                requires.append(&mut reqs);
            }
        }
        Self { requires }
    }
}

impl Setup {
    pub fn from_requirements(requirements: Requirements) -> Self {
        let install_requires = Some(requirements.requires);
        Self {
            install_requires,
            setup_requires: None,
            dev_requires: None,
            package_name: None,
            version: None,
        }
    }

    pub fn from_pyproject(pyproject: PyProject) -> Self {
        // TODO: handle extra_requires
        let (package_name, version, install_requires) = if pyproject.project.is_some() {
            let project = pyproject.project.unwrap();
            (project.name, project.version, project.dependencies)
        } else {
            (None, None, None)
        };
        let setup_requires = if pyproject.build_system.is_some() {
            let build_system = pyproject.build_system.unwrap();
            build_system.requires
        } else {
            None
        };
        Self {
            package_name,
            version,
            install_requires,
            setup_requires,
            dev_requires: None,
        }
    }
}
impl PyProject {
    pub fn from_requirements(requirements: Requirements) -> Self {
        let dependencies = Some(requirements.requires);
        let build_system = None;
        let project = Some(Project {
            name: None,
            version: None,
            dependencies,
        });
        Self {
            project,
            build_system,
        }
    }

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
