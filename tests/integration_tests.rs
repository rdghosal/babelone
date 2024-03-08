use babelone::{generators::*, parsers::*, specs::*};
use std::{env, path::Path};

#[test]
fn setup_to_requirements() {
    let curr_dir = env::current_dir().unwrap();
    let in_path = format!("{}/tests/inputs/setup.py", curr_dir.to_str().unwrap());
    let out_path = format!(
        "{}/tests/outputs/requirements__setup_to_requirements.txt",
        curr_dir.to_str().unwrap()
    );
    let setup = SetupParser::from_file(&Path::new(&in_path));
    let requirements = Requirements::from_setup(setup.unwrap());
    let result = RequirementsGenerator::make_file(&Path::new(&out_path), &requirements);
    assert!(result.is_ok());
}

#[test]
fn pyproject_to_requirements() {
    let curr_dir = env::current_dir().unwrap();
    let in_path = format!("{}/tests/inputs/pyproject.toml", curr_dir.to_str().unwrap());
    let out_path = format!(
        "{}/tests/outputs/requirements__pyproject_to_requirements.txt",
        curr_dir.to_str().unwrap()
    );
    let pyproject = PyProjectParser::from_file(&Path::new(&in_path));
    let requirements = Requirements::from_pyproject(pyproject.unwrap());
    let result = RequirementsGenerator::make_file(&Path::new(&out_path), &requirements);
    assert!(result.is_ok());
}

#[test]
fn requirements_to_setup() {
    let curr_dir = env::current_dir().unwrap();
    let in_path = format!(
        "{}/tests/inputs/requirements.txt",
        curr_dir.to_str().unwrap()
    );
    let out_path = format!(
        "{}/tests/outputs/setup__requirements_to_setup.py",
        curr_dir.to_str().unwrap()
    );
    let requirements = RequirementsParser::from_file(&Path::new(&in_path));
    let setup = Setup::from_requirements(requirements.unwrap());
    let result = SetupGenerator::make_file(&Path::new(&out_path), &setup);
    assert!(result.is_ok());
}

#[test]
fn pyproject_to_setup() {
    let curr_dir = env::current_dir().unwrap();
    let in_path = format!("{}/tests/inputs/pyproject.toml", curr_dir.to_str().unwrap());
    let out_path = format!(
        "{}/tests/outputs/setup__pyproject_to_setup.py",
        curr_dir.to_str().unwrap()
    );
    let pyproject = PyProjectParser::from_file(&Path::new(&in_path));
    let setup = Setup::from_pyproject(pyproject.unwrap());
    let result = SetupGenerator::make_file(&Path::new(&out_path), &setup);
    assert!(result.is_ok());
}

#[test]
fn requirements_to_pyproject() {
    let curr_dir = env::current_dir().unwrap();
    let in_path = format!(
        "{}/tests/inputs/requirements.txt",
        curr_dir.to_str().unwrap()
    );
    let out_path = format!(
        "{}/tests/outputs/pyproject__requirements_to_pyproject.toml",
        curr_dir.to_str().unwrap()
    );
    let requirements = RequirementsParser::from_file(&Path::new(&in_path));
    let pyproject = PyProject::from_requirements(requirements.unwrap());
    let result = PyProjectGenerator::make_file(&Path::new(&out_path), &pyproject);
    assert!(result.is_ok());
}

#[test]
fn setup_to_pyproject() {
    let curr_dir = env::current_dir().unwrap();
    let in_path = format!("{}/tests/inputs/setup.py", curr_dir.to_str().unwrap());
    let out_path = format!(
        "{}/tests/outputs/pyproject__setup_to_pyproject.toml",
        curr_dir.to_str().unwrap()
    );
    let setup = SetupParser::from_file(&Path::new(&in_path));
    let pyproject = PyProject::from_setup(setup.unwrap());
    let result = PyProjectGenerator::make_file(&Path::new(&out_path), &pyproject);
    assert!(result.is_ok());
}
