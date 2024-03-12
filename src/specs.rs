//! Models encapsulating Python package build specifications.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, default::Default, fmt};

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
#[derive(Default)]
pub struct Requirements {
    pub requires: Vec<Requirement>,
}

/// Encapsulates build specifications defined in a setup.py file.
pub struct Setup {
    pub package_name: Option<String>,
    pub version: Option<String>,
    pub extra_requires: Option<BTreeMap<String, Vec<Requirement>>>,
    pub install_requires: Option<Vec<Requirement>>,
    pub setup_requires: Option<Vec<Requirement>>,
    pub entry_points: Option<Entrypoints>,
}

pub struct Entrypoints {
    pub console_scripts: Option<Vec<String>>,
    pub gui_scripts: Option<Vec<String>>,
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
    pub requires: Option<Vec<Requirement>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: Option<String>,
    pub version: Option<String>,
    pub dependencies: Option<Vec<Requirement>>,
    #[serde(rename = "optional-dependencies")]
    pub optional_dependencies: Option<BTreeMap<String, Vec<Requirement>>>,
    #[serde(rename = "scripts")]
    pub project_scripts: Option<BTreeMap<String, String>>,
    #[serde(rename = "gui-scripts")]
    pub project_gui_scripts: Option<BTreeMap<String, String>>,
}

impl Requirements {
    pub fn from_setup(setup: Setup) -> Self {
        let mut requires = Vec::<String>::new();
        if let Some(mut install_requires) = setup.install_requires {
            requires.append(&mut install_requires);
        }
        if let Some(mut setup_requires) = setup.setup_requires {
            requires.append(&mut setup_requires);
        }
        if let Some(mut extra_requires) = setup.extra_requires {
            for mut extra_require in extra_requires.values_mut() {
                requires.append(&mut extra_require);
            }
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
            extra_requires: None,
            entry_points: None,
            package_name: None,
            version: None,
        }
    }

    pub fn from_pyproject(pyproject: PyProject) -> Self {
        let (package_name, version, install_requires, extra_requires, entry_points) =
            if pyproject.project.is_some() {
                let project = pyproject.project.unwrap();
                let mut console_scripts: Option<Vec<String>> = None;
                let mut gui_scripts: Option<Vec<String>> = None;
                if let Some(project_scripts) = project.project_scripts {
                    let mut scripts = Vec::<String>::new();
                    for (key, value) in project_scripts.iter() {
                        scripts.push(format!("{} = {}", key, value));
                    }
                    if !scripts.is_empty() {
                        console_scripts = Some(scripts);
                    }
                }
                if let Some(project_gui_scripts) = project.project_gui_scripts {
                    let mut scripts = Vec::<String>::new();
                    for (key, value) in project_gui_scripts.iter() {
                        scripts.push(format!("{} = {}", key, value));
                    }
                    if !scripts.is_empty() {
                        gui_scripts = Some(scripts);
                    }
                }
                let entry_points = if console_scripts.is_some() || gui_scripts.is_some() {
                    Some(Entrypoints {
                        console_scripts,
                        gui_scripts,
                    })
                } else {
                    None
                };
                (
                    project.name,
                    project.version,
                    project.dependencies,
                    project.optional_dependencies,
                    entry_points,
                )
            } else {
                (None, None, None, None, None)
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
            extra_requires,
            entry_points,
        }
    }
}

impl Default for Setup {
    fn default() -> Self {
        Self {
            package_name: Some(String::default()),
            version: Some(String::default()),
            entry_points: Some(Entrypoints::default()),
            extra_requires: Some(BTreeMap::default()),
            install_requires: Some(Vec::default()),
            setup_requires: Some(Vec::default()),
        }
    }
}

impl Default for Entrypoints {
    fn default() -> Self {
        Self {
            console_scripts: Some(Vec::default()),
            gui_scripts: Some(Vec::default()),
        }
    }
}

impl fmt::Debug for Entrypoints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entry(
                &"console_scripts",
                &self
                    .console_scripts
                    .as_ref()
                    .unwrap_or(&Vec::<String>::new()),
            )
            .entry(
                &"gui_scripts",
                &self.gui_scripts.as_ref().unwrap_or(&Vec::<String>::new()),
            )
            .finish()
    }
}

impl PyProject {
    pub fn from_requirements(requirements: Requirements) -> Self {
        let dependencies = Some(requirements.requires);
        let build_system = None;
        let project = Some(Project {
            dependencies,
            name: None,
            version: None,
            optional_dependencies: None,
            project_scripts: None,
            project_gui_scripts: None,
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
        let optional_dependencies = setup.extra_requires;
        let build_system = if requires.is_some() {
            Some(BuildSystem {
                requires,
                build_backend: None, // TODO
            })
        } else {
            None
        };
        let mut project_scripts: Option<BTreeMap<String, String>> = None;
        let mut project_gui_scripts: Option<BTreeMap<String, String>> = None;
        if let Some(entry_points) = setup.entry_points {
            if let Some(console_scripts) = entry_points.console_scripts {
                let mut scripts = BTreeMap::<String, String>::new();
                for console_script in console_scripts.iter() {
                    let mut key_and_path = console_script.split('=').map(|s| s.trim().to_string());
                    scripts.insert(key_and_path.next().unwrap(), key_and_path.next().unwrap());
                }
                if !scripts.is_empty() {
                    project_scripts = Some(scripts);
                }
            }
            if let Some(gui_scripts) = entry_points.gui_scripts {
                let mut scripts = BTreeMap::<String, String>::new();
                for gui_script in gui_scripts.iter() {
                    let mut key_and_path = gui_script.split('=').map(|s| s.trim().to_string());
                    scripts.insert(key_and_path.next().unwrap(), key_and_path.next().unwrap());
                }
                if !scripts.is_empty() {
                    project_gui_scripts = Some(scripts);
                }
            }
        }
        let project = Some(Project {
            name,
            version,
            dependencies,
            optional_dependencies,
            project_scripts,
            project_gui_scripts,
        });
        Self {
            project,
            build_system,
        }
    }
}

impl Default for PyProject {
    fn default() -> Self {
        Self {
            project: Some(Project::default()),
            build_system: Some(BuildSystem::default()),
        }
    }
}

impl Default for Project {
    fn default() -> Self {
        Self {
            name: Some(String::default()),
            version: Some(String::new()),
            dependencies: Some(Vec::new()),
            optional_dependencies: Some(BTreeMap::default()),
            project_scripts: Some(BTreeMap::default()),
            project_gui_scripts: Some(BTreeMap::default()),
        }
    }
}

impl Default for BuildSystem {
    fn default() -> Self {
        Self {
            build_backend: Some(String::default()),
            requires: Some(Vec::default()),
        }
    }
}
