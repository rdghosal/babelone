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
        let lines = read_file(&path)?;
        let lines = lines.split("\n").map(|s| s.to_string());
        for line in lines {
            if line.is_empty() {
                continue;
            }
            let mut split = line.split("==").map(|s| s.trim());
            let name = split.next().unwrap();
            dbg!("{}", &name);
            let version = split.next();
            requires.push((name.to_string(), version.map(|v| v.trim().to_string())));
        }
        Ok(Self { requires })
    }
}

#[derive(Debug)]
struct SetupPy {
    package_name: String,
    version: Option<String>,
    dev_requires: Option<Vec<Dependency>>,
    install_requires: Option<Vec<Dependency>>,
    setup_requires: Option<Vec<Dependency>>,
}

impl SetupPy {
    fn get_dep_from_setup(
        setup_kwargs: &BTreeMap<String, String>,
        kw: &str,
    ) -> Option<Vec<Dependency>> {
        if let Some(args) = setup_kwargs.get(kw) {
            let mut deps = Vec::<Dependency>::new();
            for arg in args.split(',') {
                if arg.is_empty() {
                    continue;
                }
                let mut split = arg.split("==").map(|s| s.trim());
                let (package, version) = (
                    split.next().unwrap().to_string(),
                    split.next().map(|s| s.to_string()),
                );
                deps.push((package, version))
            }
            Some(deps)
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
        let contents = read_file(&path)?.replace('\n', "");
        let re = Regex::new(r".*setup\((.*)\)").unwrap();
        let Some(setup) = re.captures(&contents) else {
            return Err(Box::new(PyValueError::new_err(
                "Failed to parse setup.py. Invocation of a `setup` callable not found.",
            )));
        };
        dbg!(re.captures(&contents).unwrap()[1].to_string());
        let mut kwargs = BTreeMap::<String, String>::new();
        let (mut kw, mut arg) = (String::new(), String::new());
        let (mut kw_done, mut is_list_arg) = (false, false);
        for char in setup[1].replace('"', "").replace("'", "").chars() {
            match (char, kw_done) {
                ('[', true) => is_list_arg = true,
                (']', true) => is_list_arg = false,
                (',', true) if is_list_arg => {
                    arg.push(char);
                }
                (',', true) if !is_list_arg => {
                    kwargs.insert(kw.trim().to_string(), arg.trim().to_string());
                    kw.clear();
                    arg.clear();
                    (kw_done, is_list_arg) = (false, false);
                }
                ('=', false) => kw_done = true,
                (_, true) => {
                    arg.push(char);
                }
                (_, false) => {
                    kw.push(char);
                }
            }
        }
        dbg!("{}", &kwargs);

        let Some(package_name) = kwargs.get("name") else {
            return Err(Box::new(PyValueError::new_err(
                "Failed to parse required `package_name` from setup.",
            )));
        };
        let package_name = package_name.trim().to_string();
        let version = kwargs.get("version").map(|v| v.to_string());
        let install_requires = Self::get_dep_from_setup(&kwargs, "install_requires");
        let dev_requires = Self::get_dep_from_setup(&kwargs, "dev_requires");
        let setup_requires = Self::get_dep_from_setup(&kwargs, "setup_requires");
        Ok(Self {
            package_name,
            version,
            install_requires,
            dev_requires,
            setup_requires,
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

    #[test]
    fn make_setuppy() {
        let curr_dir = env::current_dir().unwrap();
        let path_str = format!("{}/tests/static/setup.py", curr_dir.to_str().unwrap());
        let path = Path::new(&path_str);
        let s = SetupPy::from_file(&path).unwrap();
        dbg!(&s);
        assert_eq!(s.package_name, "babelone-test");
        assert_eq!(s.version, Some("2.0".to_string()));
        assert_eq!(s.dev_requires, None);
        assert_eq!(
            s.install_requires,
            Some(vec![
                ("pydantic".to_string(), Some("2.6.2".to_string())),
                ("fastapi".to_string(), None)
            ])
        );
    }
}
