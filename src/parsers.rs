//! Defines parsers used to exract Python package build specifications
//! from applicable file types, e.g., requirements.txt, setup.py, and
//! pyproject.toml

use crate::utils;
use pyo3::exceptions::PyValueError;
use rustpython_parser::{ast, Parse};
use serde::Deserialize;
use std::{collections::BTreeMap, error::Error, path::Path};

/// Denotes a Python package dependency and its required version,
///
/// # Examples
/// `"pydantic==2.x"`, `"flask<3.0"`
type Requirement = String;

/// A build specification for a Python package, e.g., setup.py.
trait PyBuildSpec {
    fn from_file(path: &Path) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}

trait PyStr {
    fn to_string(&self) -> Result<String, pyo3::PyErr>;
}

trait PyStrList {
    fn to_string_vec(&self) -> Result<Vec<String>, pyo3::PyErr>;
}

trait IdentValueMap {
    fn insert_assignments(&mut self, assignment: &ast::StmtAssign) -> &mut Self;
}

/// Encapsulates build requirements defined in a requirements.txt (or similar file).
struct Requirements {
    requires: Vec<Requirement>,
}

impl PyBuildSpec for Requirements {
    fn from_file(path: &Path) -> Result<Self, Box<dyn Error>> {
        let mut requires = Vec::<Requirement>::new();
        let lines = utils::read_file(&path)?;
        let lines = lines.split("\n").map(|s| s.to_string());
        for line in lines {
            if line.is_empty() {
                continue;
            }
            requires.push(line.trim().replace(" ", "").to_string());
        }
        Ok(Self { requires })
    }
}

/// Encapsulates build specifications defined in a setup.py file.
struct Setup {
    package_name: String,
    version: Option<String>,
    dev_requires: Option<Vec<Requirement>>,
    install_requires: Option<Vec<Requirement>>,
    setup_requires: Option<Vec<Requirement>>,
}

impl Setup {
    fn parse_string(
        expr: &ast::Expr,
        assignments: &BTreeMap<String, ast::Expr>,
    ) -> Result<String, pyo3::PyErr> {
        match expr {
            ast::Expr::Constant(_) => {
                return Ok(expr.to_string()?);
            }
            ast::Expr::Name(name) => {
                if let Some(v) = assignments.get(&name.id.to_string()) {
                    return Ok(v.to_string()?);
                }
            }
            _ => (),
        }
        return Err(PyValueError::new_err("Failed to parse string."));
    }

    fn parse_string_vec(
        expr: &ast::Expr,
        assignments: &BTreeMap<String, ast::Expr>,
    ) -> Result<Vec<String>, pyo3::PyErr> {
        match expr {
            ast::Expr::List(_) => {
                return Ok(expr.to_string_vec()?);
            }
            ast::Expr::Name(name) => {
                if let Some(v) = assignments.get(&name.id.to_string()) {
                    return Ok(v.to_string_vec()?);
                }
            }
            _ => (),
        }
        return Err(PyValueError::new_err("Failed to parse Vec<String>."));
    }

    fn get_setup_call<'a>(
        statements: &'a Vec<ast::Stmt>,
        idx: &mut usize,
        assignments: &'a mut BTreeMap<String, ast::Expr>,
    ) -> Option<(&'a ast::ExprCall, &'a mut BTreeMap<String, ast::Expr>)> {
        if *idx < statements.len() {
            match &statements[*idx] {
                ast::Stmt::Assign(assignment) => {
                    assignments.insert_assignments(assignment);
                }
                ast::Stmt::If(if_stmt) => {
                    return Self::get_setup_call(&if_stmt.body, &mut 0, assignments);
                }
                ast::Stmt::Expr(expr) => {
                    if let ast::Expr::Call(c) = expr.value.as_ref() {
                        let is_setup = match c.func.as_ref() {
                            ast::Expr::Name(n) => "setup" == n.id.as_str(),
                            ast::Expr::Attribute(a) => "setup" == a.attr.as_str(),
                            _ => false,
                        };
                        if is_setup {
                            return Some((c, assignments));
                        }
                    };
                }
                _ => (),
            };
            *idx += 1;
            return Self::get_setup_call(statements, idx, assignments);
        }
        return None;
    }
}

impl PyBuildSpec for Setup {
    fn from_file(path: &Path) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let contents = utils::read_file(&path)?;
        let mut assignments = BTreeMap::<String, ast::Expr>::new();
        let statements = ast::Suite::parse(&contents, &path.to_str().unwrap())?;

        let mut package_name: Option<String> = None;
        let mut version: Option<String> = None;
        let mut install_requires: Option<Vec<Requirement>> = None;
        let mut setup_requires: Option<Vec<Requirement>> = None;
        let mut dev_requires: Option<Vec<Requirement>> = None;

        if let Some((setup, assignments)) =
            Self::get_setup_call(&statements, &mut 0, &mut assignments)
        {
            for keyword in &setup.keywords {
                let ident = keyword.arg.clone().unwrap();
                match ident.as_str() {
                    "name" => {
                        package_name = Some(Self::parse_string(&keyword.value, &assignments)?)
                    }
                    "version" => version = Some(Self::parse_string(&keyword.value, &assignments)?),
                    "install_requires" => {
                        install_requires =
                            Some(Self::parse_string_vec(&keyword.value, &assignments)?);
                    }
                    "setup_requires" => {
                        setup_requires =
                            Some(Self::parse_string_vec(&keyword.value, &assignments)?);
                    }
                    "dev_requires" => {
                        dev_requires = Some(Self::parse_string_vec(&keyword.value, &assignments)?);
                    }
                    _ => continue,
                }
            }
        }
        if package_name.is_none() {
            return Err(Box::new(PyValueError::new_err(
                "package_name must be defined.",
            )));
        }
        Ok(Self {
            package_name: package_name.unwrap(),
            version,
            install_requires,
            dev_requires,
            setup_requires,
        })
    }
}
impl PyStr for ast::Expr {
    fn to_string(&self) -> Result<String, pyo3::PyErr> {
        if let ast::Expr::Constant(c) = &self {
            if let ast::Constant::Str(s) = &c.value {
                return Ok(s.clone());
            }
        }
        return Err(PyValueError::new_err(
            "Failed to parse String value from ExprConstant.",
        ));
    }
}

impl PyStrList for ast::Expr {
    fn to_string_vec(&self) -> Result<Vec<String>, pyo3::PyErr> {
        if let ast::Expr::List(list) = &self {
            let mut result = Vec::<String>::new();
            for element in &list.elts {
                if let ast::Expr::Constant(c) = element {
                    if let ast::Constant::Str(s) = &c.value {
                        result.push(s.clone());
                    }
                }
            }
            return Ok(result);
        }
        return Err(PyValueError::new_err(
            "Failed to parse Expr as Vec<String>.",
        ));
    }
}

impl IdentValueMap for BTreeMap<String, ast::Expr> {
    fn insert_assignments(&mut self, assignment: &ast::StmtAssign) -> &mut Self {
        let mut identifiers = Vec::<String>::new();
        for target in assignment.targets.iter() {
            let ast::Expr::Name(e) = target else {
                panic!("Expected name of Expr::Name in assignment parsing.");
            };
            let identifier = e.id.to_string();
            identifiers.push(identifier);
        }
        for identifier in identifiers {
            self.insert(identifier, *assignment.value.clone());
        }
        self
    }
}

/// Encapsulates build specifications defined in a pyproject.toml file.
#[derive(Deserialize)]
struct PyProject {
    #[serde(rename = "build-system")]
    build_system: Option<BuildSystem>,
    project: Option<Project>,
}

#[derive(Deserialize)]
struct BuildSystem {
    requires: Option<Vec<String>>,
    #[serde(rename = "build-backend")]
    build_backend: Option<String>,
}

#[derive(Deserialize)]
struct Project {
    name: String,
    version: String,
    dependencies: Option<Vec<String>>,
}

impl PyBuildSpec for PyProject {
    fn from_file(path: &Path) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let contents = utils::read_file(&path)?;
        let pyproject = toml::from_str::<Self>(&contents)?;
        Ok(pyproject)
    }
}

#[cfg(test)]
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
            vec!["flask".to_string(), "pydantic==2.x".to_string()]
        );
    }

    #[test]
    fn make_setuppy() {
        let curr_dir = env::current_dir().unwrap();
        let path_str = format!("{}/tests/static/setup.py", curr_dir.to_str().unwrap());
        let path = Path::new(&path_str);
        let s = Setup::from_file(&path).unwrap();
        assert_eq!(s.package_name, "babelone-test");
        assert_eq!(s.version, Some("2.0".to_string()));
        assert_eq!(s.dev_requires, None);
        assert_eq!(s.setup_requires, None);
        assert_eq!(
            s.install_requires,
            Some(vec!["pydantic==2.6.2".to_string(), "fastapi".to_string(),])
        );
    }

    #[test]
    fn make_pyproject() {
        let curr_dir = env::current_dir().unwrap();
        let path_str = format!("{}/tests/static/pyproject.toml", curr_dir.to_str().unwrap());
        let path = Path::new(&path_str);
        let p = PyProject::from_file(&path).unwrap();
        let build_system = p.build_system.unwrap();
        let project = p.project.unwrap();
        assert_eq!(&build_system.requires, &Some(vec!["hatchling".to_string()]));
        assert_eq!(
            &build_system.build_backend,
            &Some("hatchling.build".to_string())
        );
        assert_eq!(&project.name, "spam-eggs");
        assert_eq!(&project.version, "2020.0.0");
        assert_eq!(
            &project.dependencies,
            &Some(vec![
                "httpx".to_string(),
                "gidgethub[httpx]>4.0.0".to_string(),
                "django>2.1; os_name != 'nt'".to_string(),
                "django>2.0; os_name == 'nt'".to_string(),
            ])
        );
    }
}
