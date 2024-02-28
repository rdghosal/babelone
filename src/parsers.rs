use std::{fs, path::Path, error::Error};

type Dependency = (String, Option<String>);

trait BuildSpecFile {
    fn from_file(path: &Path) -> Result<Self, Box<dyn Error>> where Self: Sized;
}

struct Requirements {
    requires: Vec<Dependency>,
}

impl BuildSpecFile for Requirements {
    fn from_file(path: &Path) -> Result<Self, Box<dyn Error>> {
        let contents = read_file(path);
        let mut requires = Vec::<Dependency>::new();
        for line in contents?.split('\n') {
            if line.is_empty() {
                continue;
            }
            let mut split = line.split("==");
            let name = split.next().unwrap();
            let version = split.next();
            requires.push((name.to_string(), version.map(|v| v.to_string())));
        }
        Ok(Self {requires})
    }

}

struct SetupPy {
    package_name: String,
    version: Option<String>,
    dev_requires: Vec<Dependency>,
    requires: Vec<Dependency>,
    setup_requires: Vec<Dependency>,
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
    use std::env;
    use super::*;

    #[test]
    fn make_requirments() {
        let curr_dir = env::current_dir().unwrap();
        let path_str = format!("{}/tests/static/requirements.txt", curr_dir.to_str().unwrap());
        let path = Path::new(&path_str);
        let r = Requirements::from_file(&path).unwrap();
        assert_eq!(r.requires, vec![("flask".to_string(), None), ("pydantic".to_string(), Some("2.x".to_string()))]);
    }
}
