use pyo3::exceptions::PyOSError;
use pyo3::PyResult;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::specs::*;

pub struct RequirementsGenerator;
pub struct SetupGenerator;
pub struct PyProjectGenerator;

pub trait SpecGenerator<T> {
    fn make_file(path: &Path, spec: &T) -> PyResult<()>;
}

trait SetupKwarg {
    fn as_kwarg_string(&self, kw: &str) -> String;
}

impl SpecGenerator<Requirements> for RequirementsGenerator {
    fn make_file(path: &Path, spec: &Requirements) -> PyResult<()> {
        let mut contents = String::new();
        for requirement in spec.requires.iter() {
            contents.push_str(&requirement);
            contents.push_str("\n");
        }
        fs::write(path, contents)?;
        Ok(())
    }
}

impl SpecGenerator<Setup> for SetupGenerator {
    fn make_file(path: &Path, spec: &Setup) -> PyResult<()> {
        let mut contents = String::new();
        let docstring_end = if spec.package_name.as_ref().is_some_and(|s| !s.is_empty()) {
            format!(" for {}", &spec.package_name.as_ref().unwrap())
        } else {
            String::new()
        };
        let docstring = format!(
            r#""""Installation configuration and package metadata{}.""""#,
            docstring_end
        );
        let imports = "from setuptools import setup";
        let mut setup_call = String::from("    setup(\n");
        let kwargs: Vec<String> = vec![
            spec.package_name.as_kwarg_string("package_name"),
            spec.version.as_kwarg_string("version"),
            spec.install_requires.as_kwarg_string("install_requires"),
            spec.setup_requires.as_kwarg_string("setup_requires"),
            spec.extra_requires.as_kwarg_string("extra_requires"),
            spec.entry_points.as_kwarg_string("entry_points"),
        ];
        for kwarg in kwargs.iter() {
            if kwarg.is_empty() {
                continue;
            }
            let formatted = format!("        {},\n", kwarg);
            setup_call.push_str(&formatted);
        }
        setup_call.push_str("    )");
        let entrypoint = r#"if __name__ == "__main__":"#;
        contents.push_str(&docstring);
        contents.push_str("\n");
        contents.push_str(&imports);
        contents.push_str("\n\n\n");
        contents.push_str(&entrypoint);
        contents.push_str("\n");
        contents.push_str(&setup_call);
        fs::write(path, contents)?;
        Ok(())
    }
}

impl SetupKwarg for Option<String> {
    fn as_kwarg_string(&self, kw: &str) -> String {
        match self {
            Some(s) => format!("{}={:#?}", kw, s),
            None => String::new(),
        }
    }
}

impl SetupKwarg for Option<Vec<String>> {
    fn as_kwarg_string(&self, kw: &str) -> String {
        match self {
            Some(s) => format!("{}={:?}", kw, s),
            None => String::new(),
        }
    }
}

impl SetupKwarg for Option<BTreeMap<String, Vec<Requirement>>> {
    fn as_kwarg_string(&self, kw: &str) -> String {
        match self {
            Some(s) => format!("{}={:?}", kw, s),
            None => String::new(),
        }
    }
}

impl SetupKwarg for Option<Entrypoints> {
    fn as_kwarg_string(&self, kw: &str) -> String {
        match self {
            Some(s) => format!("{}={:?}", kw, s),
            None => String::new(),
        }
    }
}

impl SpecGenerator<PyProject> for PyProjectGenerator {
    fn make_file(path: &Path, spec: &PyProject) -> PyResult<()> {
        if let Ok(contents) = toml::to_string_pretty::<PyProject>(&spec) {
            fs::write(path, contents)?;
            return Ok(());
        }
        Err(PyOSError::new_err(format!(
            "Failed to write {:#?} with pyproject definition:\n{:#?}",
            path.to_str(),
            spec
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn generate_requirements() {
        let curr_dir = env::current_dir().unwrap();
        let path_str = format!(
            "{}/tests/outputs/requirements__generate_requirements.txt",
            curr_dir.to_str().unwrap()
        );
        let path = Path::new(&path_str);
        let spec = Requirements {
            requires: vec!["flask".to_string(), "pydantic==2.6.1".to_string()],
        };
        let result = RequirementsGenerator::make_file(&path, &spec);
        assert!(result.is_ok());
    }

    #[test]
    fn generate_setup() {
        let curr_dir = env::current_dir().unwrap();
        let path_str = format!(
            "{}/tests/outputs/setupy__generate_setup.py",
            curr_dir.to_str().unwrap()
        );
        let path = Path::new(&path_str);
        let spec = Setup {
            package_name: Some("babelone-test".to_string()),
            version: Some("v0.1.1".to_string()),
            extra_requires: Some(BTreeMap::from([(
                "dev".to_string(),
                vec!["pytest".to_string(), "hypothesis>=6.98.1".to_string()],
            )])),
            install_requires: Some(vec!["flask".to_string(), "pydantic==2.6.1".to_string()]),
            setup_requires: None,
            entry_points: Some(Entrypoints {
                console_scripts: Some(vec!["hello-world = timmins:hello_world".to_string()]),
                gui_scripts: None,
            }),
        };
        let result = SetupGenerator::make_file(&path, &spec);
        assert!(result.is_ok());
    }

    #[test]
    fn generate_pyproject() {
        let curr_dir = env::current_dir().unwrap();
        let path_str = format!(
            "{}/tests/outputs/pyproject__generate_pyproject.toml",
            curr_dir.to_str().unwrap()
        );
        let path = Path::new(&path_str);
        let spec = PyProject {
            project: Some(Project {
                name: Some("test".to_string()),
                version: Some("2.1".to_string()),
                dependencies: Some(vec!["pydantic==2.x".to_string(), "flask".to_string()]),
                optional_dependencies: Some(BTreeMap::from([(
                    "dev".to_string(),
                    vec!["pytest".to_string(), "hypothesis>=6.98.1".to_string()],
                )])),
                project_scripts: None,
                project_gui_scripts: None,
            }),
            build_system: None,
        };
        let result = PyProjectGenerator::make_file(&path, &spec);
        assert!(result.is_ok());
    }
}
