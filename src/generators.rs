use std::error::Error;
use std::fs;
use std::path::Path;

use crate::specs::*;

pub struct PyProjectGenerator;

pub trait SpecGenerator<T> {
    fn make_file(path: &Path, spec: &T) -> Result<(), Box<dyn Error>>;
}

impl SpecGenerator<PyProject> for PyProjectGenerator {
    fn make_file(path: &Path, spec: &PyProject) -> Result<(), Box<dyn Error>> {
        let contents = toml::to_string::<PyProject>(&spec)?;
        fs::write(path, contents)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

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
                name: "test".to_string(),
                version: Some("2.1".to_string()),
                dependencies: Some(vec!["pydantic==2.x".to_string(), "flask".to_string()]),
            }),
            build_system: None,
        };
        let result = PyProjectGenerator::make_file(&path, &spec);
        assert!(result.is_ok());
    }
}
