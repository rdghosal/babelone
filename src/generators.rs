use std::error::Error;
use std::fs;
use std::path::Path;

use crate::specs::*;

pub struct RequirementsGenerator;
pub struct SetupGenerator;
pub struct PyProjectGenerator;

pub trait SpecGenerator<T> {
    fn make_file(path: &Path, spec: &T) -> Result<(), Box<dyn Error>>;
}

impl SpecGenerator<Requirements> for RequirementsGenerator {
    fn make_file(path: &Path, spec: &Requirements) -> Result<(), Box<dyn Error>> {
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
    fn make_file(path: &Path, spec: &Setup) -> Result<(), Box<dyn Error>> {
        let mut contents = String::new();
        let docstring_end = if spec.package_name.is_some() {
            format!("for {}", &spec.package_name.as_ref().unwrap())
        } else {
            String::new()
        };
        let docstring = format!(
            r#""""Installation configuration and package metadata {}.""""#,
            docstring_end
        );
        let imports = "from setuptools import setup";
        // TODO: handle extra_requires>
        let setup_call = format!(
            r#"
    setup(
        package_name={:#?},
        version={:#?},
        install_requires={:?},
        setup_requires={:?},
    )"#,
            spec.package_name.as_ref().unwrap_or(&String::new()),
            spec.version.as_ref().unwrap_or(&String::new()),
            spec.install_requires
                .as_ref()
                .unwrap_or(&Vec::<String>::new()),
            spec.setup_requires
                .as_ref()
                .unwrap_or(&Vec::<String>::new())
        );
        let entrypoint = r#"if __name__ == "__main__":"#;
        contents.push_str(&docstring);
        contents.push_str("\n");
        contents.push_str(&imports);
        contents.push_str("\n\n\n");
        contents.push_str(&entrypoint);
        contents.push_str(&setup_call);
        fs::write(path, contents)?;
        Ok(())
    }
}

impl SpecGenerator<PyProject> for PyProjectGenerator {
    fn make_file(path: &Path, spec: &PyProject) -> Result<(), Box<dyn Error>> {
        let contents = toml::to_string_pretty::<PyProject>(&spec)?;
        fs::write(path, contents)?;
        Ok(())
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
            dev_requires: None,
            install_requires: Some(vec!["flask".to_string(), "pydantic==2.6.1".to_string()]),
            setup_requires: None,
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
            }),
            build_system: None,
        };
        let result = PyProjectGenerator::make_file(&path, &spec);
        assert!(result.is_ok());
    }
}
