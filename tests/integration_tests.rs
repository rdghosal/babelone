use babelone::{
    generators::{PyProjectGenerator, SpecGenerator},
    parsers::{SetupParser, SpecParser},
    specs::PyProject,
};
use std::{env, path::Path};

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
