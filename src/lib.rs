use generators::SpecGenerator;
use parsers::SpecParser;
use pyo3::{
    exceptions::{PyNotImplementedError, PyValueError},
    prelude::*,
};
use std::path::Path;

pub mod generators;
pub mod parsers;
pub mod specs;
mod utils;

fn get_spec_type(path: &Path) -> PyResult<specs::PyBuildSpec> {
    if let Some(file_name) = path.file_name() {
        if let Some(file_name) = file_name.to_str() {
            let t = match file_name {
                "requirements.txt" => Some(specs::PyBuildSpec::Requirements),
                "setup.py" => Some(specs::PyBuildSpec::Setup),
                "pyproject.toml" => Some(specs::PyBuildSpec::PyProject),
                _ => None,
            };
            if t.is_some() {
                return Ok(t.unwrap());
            }
        }
    }
    return Err(PyValueError::new_err(
        "Failed to parse filename. Must be one of: requirements.txt, setup.py, pyproject.toml.",
    ));
}

/// Scaffolds a build specification file.
#[pyfunction]
fn create(destination: String) -> PyResult<()> {
    let destination = Path::new(&destination);
    let dest_type = get_spec_type(&destination)?;
    match dest_type {
        specs::PyBuildSpec::Requirements => {
            let requirements = specs::Requirements::default();
            generators::RequirementsGenerator::make_file(&destination, &requirements)?;
            Ok(())
        }
        specs::PyBuildSpec::Setup => {
            let setup = specs::Setup::default();
            generators::SetupGenerator::make_file(&destination, &setup)?;
            Ok(())
        }
        specs::PyBuildSpec::PyProject => {
            let pyproject = specs::PyProject::default();
            generators::PyProjectGenerator::make_file(&destination, &pyproject)?;
            Ok(())
        }
    }
}

/// Transpiles a source Python package build specification file (e.g., setup.py)
/// to another (e.g., pyproject.toml).
#[pyfunction]
fn translate(source: String, destination: String) -> PyResult<()> {
    let source = Path::new(&source);
    let destination = Path::new(&destination);
    let source_type = get_spec_type(&source)?;
    let dest_type = get_spec_type(&destination)?;
    match (source_type, dest_type) {
        (specs::PyBuildSpec::Requirements, specs::PyBuildSpec::PyProject) => {
            let requirements = parsers::RequirementsParser::from_file(&source)?;
            let pyproject = specs::PyProject::from_requirements(requirements);
            generators::PyProjectGenerator::make_file(&destination, &pyproject)?;
            Ok(())
        }
        (specs::PyBuildSpec::Setup, specs::PyBuildSpec::PyProject) => {
            let setup = parsers::SetupParser::from_file(&source)?;
            let pyproject = specs::PyProject::from_setup(setup);
            generators::PyProjectGenerator::make_file(&destination, &pyproject)?;
            Ok(())
        }
        (specs::PyBuildSpec::Requirements, specs::PyBuildSpec::Setup) => {
            let requirements = parsers::RequirementsParser::from_file(&source)?;
            let setup = specs::Setup::from_requirements(requirements);
            generators::SetupGenerator::make_file(&destination, &setup)?;
            Ok(())
        }
        (specs::PyBuildSpec::PyProject, specs::PyBuildSpec::Setup) => {
            let pyproject = parsers::PyProjectParser::from_file(&source)?;
            let setup = specs::Setup::from_pyproject(pyproject);
            generators::SetupGenerator::make_file(&destination, &setup)?;
            Ok(())
        }
        (specs::PyBuildSpec::Setup, specs::PyBuildSpec::Requirements) => {
            let setup = parsers::SetupParser::from_file(&source)?;
            let requirements = specs::Requirements::from_setup(setup);
            generators::RequirementsGenerator::make_file(&destination, &requirements)?;
            Ok(())
        }
        (specs::PyBuildSpec::PyProject, specs::PyBuildSpec::Requirements) => {
            let pyproject = parsers::PyProjectParser::from_file(&source)?;
            let requirements = specs::Requirements::from_pyproject(pyproject);
            generators::RequirementsGenerator::make_file(&destination, &requirements)?;
            Ok(())
        }
        _ => Err(PyNotImplementedError::new_err("Failed to perform operation. Only unique conversions between requirements.txt, setup.py and pyproject.toml are allowed.")),
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn _babelone_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(create, m)?)?;
    m.add_function(wrap_pyfunction!(translate, m)?)?;
    Ok(())
}
