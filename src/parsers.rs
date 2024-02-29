use pyo3::exceptions::{PySyntaxError, PyValueError};
use regex::Regex;
use std::{collections::BTreeMap, error::Error, fs, iter::Map, path::Path};

type Dependency = (String, Option<String>);

trait BuildSpecFile {
    fn from_file(path: &Path) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}

struct Requirements {
    requires: Vec<Dependency>,
}

impl BuildSpecFile for Requirements {
    fn from_file(path: &Path) -> Result<Self, Box<dyn Error>> {
        let mut requires = Vec::<Dependency>::new();
        let lines = read_file(&path)?.split("\n");
        for line in lines.map(|s| s.to_string()).into_iter() {
            if line.is_empty() {
                continue;
            }
            let mut split = line.split("==");
            let name = split.next().unwrap().trim();
            let version = split.next();
            requires.push((name.to_string(), version.map(|v| v.trim().to_string())));
        }
        Ok(Self { requires })
    }
}

struct SetupPy {
    package_name: String,
    version: Option<String>,
    dev_requires: Option<Vec<Dependency>>,
    requires: Option<Vec<Dependency>>,
    setup_requires: Option<Vec<Dependency>>,
}

impl SetupPy {
    fn get_dep_from_setup(setup_kwargs: &BTreeMap<&str, &str>, kw: &str) -> Option<Dependency> {
        if let Some(arg) = setup_kwargs.get(kw) {
            let mut split = arg.split("==");
            let (package, version) = (
                split.next().unwrap().to_string(),
                split.next().map(|s| s.to_string()),
            );
            Some((package, version))
        } else {
            None
        }
    }
}

impl BuildSpecFile for SetupPy {

    fn from_file(path: &Path) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let mut pos: usize = 0;
        let mut contents = read_file(&path)?;
        let re = Regex::new(r"setup\((.*)\)").unwrap();
        let Some(setup) = re.captures(&contents) else {
            return Err(Box::new(PyValueError::new_err(
                "Failed to parse setup.py. Invocation of a `setup` callable not found.",
            )));
        };
        // TODO: make options for key enum.
        let kwargs = BTreeMap::<&str, &str>::new();
        for kwarg in setup[1].split(',') {
            let split = kwarg.split('=');
            let (kw, arg) = (split.next().unwrap(), split.next().unwrap().trim());
            kwargs.insert(kw, arg);
        }
        let Some(package_name) = kwargs.get("name") else {
            return Err(Box::new(PyValueError::new_err(
                "Failed to parse requireed `package_name` from setup.",
            )));
        };
        let package_name = package_name.trim().to_string();
        let version = kwargs.get("version").map(|v| v.to_string());
        let requires = Self::get_dep_from_setup(&kwargs, "version");
        let dev_requires = Self::get_dep_from_setup(&kwargs, "version");
        let setup_requires = Self::get_dep_from_setup(&kwargs, "version");
        Ok(Self {
            package_name, version, requires, dev_requires, setup_requires

        })
    }
}

struct PyProject {
    build_system: Vec<String>, // TODO: type
    package_name: String,
    version: Option<String>,
    dev_requires: Vec<Dependency>,
    requires: Vec<Dependency>,
    setup_requires: Vec<Dependency>,
}

fn read_file(path: &Path) -> Result<String, Box<dyn Error>> {
    Ok(String::from_utf8(fs::read(path)?)?)
}

mod test {
    use super::*;
    use std::env;

    #[test]
    fn make_requirments() {
        let curr_dir = env::current_dir().unwrap();
        let path_str = format!(
            "{}/tests/static/requirements.txt",
            curr_dir.to_str().unwrap()
        );
        let path = Path::new(&path_str);
        let r = Requirements::from_file(&path).unwrap();
        assert_eq!(
            r.requires,
            vec![
                ("flask".to_string(), None),
                ("pydantic".to_string(), Some("2.x".to_string()))
            ]
        );
    }
}
