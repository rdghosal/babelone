//! Defines parsers used to exract Python package build specifications
//! from applicable file types, e.g., requirements.txt, setup.py, and
//! pyproject.toml
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::PyResult;
use rustpython_parser::{ast, Parse};
use std::{collections::BTreeMap, path::Path};

use crate::specs::*;
use crate::utils;

pub struct RequirementsParser;
pub struct SetupParser;
pub struct PyProjectParser;

enum PyAssignment<'a> {
    Annotated(&'a ast::StmtAnnAssign),
    Unannotated(&'a ast::StmtAssign),
}

/// A build specification for a Python package, e.g., setup.py.
pub trait SpecParser<T> {
    fn from_file(path: &Path) -> PyResult<T>
    where
        Self: Sized;
}

trait PyStr {
    fn to_string(&self) -> PyResult<String>;
}

trait PyIdent {
    fn as_ident(&self) -> PyResult<String>;
}

trait PyStrList {
    fn to_string_vec(&self) -> PyResult<Vec<String>>;
}

trait IdentValueMap {
    fn insert_assignments(&mut self, assignment: PyAssignment) -> PyResult<&mut Self>;
}

impl SpecParser<Requirements> for RequirementsParser {
    fn from_file(path: &Path) -> PyResult<Requirements> {
        let mut requires = Vec::<Requirement>::new();
        let lines = utils::read_file(&path)?;
        let lines = lines.split("\n").map(|s| s.to_string());
        for line in lines {
            if line.is_empty() {
                continue;
            }
            requires.push(line.trim().replace(" ", "").to_string());
        }
        Ok(Requirements { requires })
    }
}

impl SpecParser<Setup> for SetupParser {
    fn from_file(path: &Path) -> PyResult<Setup>
    where
        Self: Sized,
    {
        let contents = utils::read_file(&path)?;
        match ast::Suite::parse(&contents, &path.to_str().unwrap()) {
            Ok(statements) => Ok(Self::parse_ast(statements)?),
            Err(_) => Err(PyValueError::new_err(format!(
                "Failed to parse AST of {:#?}",
                path.to_str()
            ))),
        }
    }
}

impl SpecParser<PyProject> for PyProjectParser {
    fn from_file(path: &Path) -> PyResult<PyProject>
    where
        Self: Sized,
    {
        let contents = utils::read_file(&path)?;
        let pyproject = toml::from_str::<PyProject>(&contents);
        if pyproject.is_ok() {
            return Ok(pyproject.unwrap());
        }
        Err(PyValueError::new_err(format!(
            "Failed to parse toml file {:#?}",
            path.to_str()
        )))
    }
}

impl SetupParser {
    fn parse_ast(statements: Vec<ast::Stmt>) -> PyResult<Setup> {
        let mut assignments = BTreeMap::<String, ast::Expr>::new();

        let mut package_name: Option<String> = None;
        let mut version: Option<String> = None;
        let mut install_requires: Option<Vec<Requirement>> = None;
        let mut setup_requires: Option<Vec<Requirement>> = None;
        let mut extra_requires: Option<BTreeMap<String, Vec<Requirement>>> = None;
        let mut entry_points: Option<Entrypoints> = None;

        if let Some((setup, assignments)) =
            Self::get_setup_call(&statements, &mut 0, &mut assignments)?
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
                    "extra_requires" => {
                        extra_requires =
                            Some(Self::parse_requires_map(&keyword.value, &assignments)?);
                    }
                    "entry_points" => {
                        entry_points = Some(Self::parse_entrypoints(&keyword.value, &assignments)?);
                    }
                    _ => continue,
                }
            }
        }
        Ok(Setup {
            package_name,
            version,
            install_requires,
            extra_requires,
            setup_requires,
            entry_points,
        })
    }

    fn parse_string(
        expr: &ast::Expr,
        assignments: &BTreeMap<String, ast::Expr>,
    ) -> PyResult<String> {
        match expr {
            ast::Expr::Constant(_) => {
                return Ok(expr.to_string()?);
            }
            ast::Expr::Name(name) => {
                if let Some(v) = assignments.get(&name.id.to_string()) {
                    return Ok(v.to_string()?);
                }
            }
            ast::Expr::JoinedStr(joined) => {
                let mut res = String::new();
                for value in joined.values.iter() {
                    let mut target = value;
                    if let ast::Expr::FormattedValue(formatted) = value {
                        target = formatted.value.as_ref();
                    }
                    res.push_str(&Self::parse_string(target, assignments)?);
                }
                return Ok(res);
            }
            _ => (),
        }
        return Err(PyValueError::new_err(format!(
            "Failed to parse String from Expr:\n{expr:#?}",
        )));
    }

    fn parse_string_vec(
        expr: &ast::Expr,
        assignments: &BTreeMap<String, ast::Expr>,
    ) -> PyResult<Vec<String>> {
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
        return Err(PyValueError::new_err(format!(
            "Failed to parse Vec<String> from Expr:\n{expr:#?}"
        )));
    }

    fn parse_requires_map(
        expr: &ast::Expr,
        assignments: &BTreeMap<String, ast::Expr>,
    ) -> PyResult<BTreeMap<String, Vec<Requirement>>> {
        let mut mapped = BTreeMap::<String, Vec<Requirement>>::new();
        match expr {
            ast::Expr::Dict(dict) => {
                for (i, key) in dict.keys.iter().enumerate() {
                    if let Some(key) = key {
                        let value = &dict.values[i];
                        mapped.insert(
                            key.to_string()?,
                            Self::parse_string_vec(value, assignments)?,
                        );
                    }
                }
                return Ok(mapped);
            }
            ast::Expr::Name(name) => {
                if let Some(v) = assignments.get(&name.id.to_string()) {
                    return Ok(Self::parse_requires_map(v, assignments)?);
                }
            }
            _ => (),
        }
        return Err(PyValueError::new_err(format!(
            "Failed to parse BTreeMap<String, Vec<String>> from Expr:\n{expr:#?}"
        )));
    }

    fn parse_entrypoints(
        expr: &ast::Expr,
        assignments: &BTreeMap<String, ast::Expr>,
    ) -> PyResult<Entrypoints> {
        match expr {
            ast::Expr::Dict(dict) => {
                let mut entry_points = Entrypoints {
                    console_scripts: None,
                    gui_scripts: None,
                };
                for (i, key) in dict.keys.iter().enumerate() {
                    if let Some(key) = key {
                        let key = key.to_string()?;
                        if key == "console_scripts".to_string() {
                            entry_points.console_scripts = Some(dict.values[i].to_string_vec()?);
                        } else if key == "gui_scripts".to_string() {
                            entry_points.gui_scripts = Some(dict.values[i].to_string_vec()?);
                        }
                    }
                }
                if entry_points.console_scripts.is_some() || entry_points.gui_scripts.is_some() {
                    return Ok(entry_points);
                }
            }
            ast::Expr::Name(name) => {
                if let Some(v) = assignments.get(&name.id.to_string()) {
                    return Ok(Self::parse_entrypoints(v, assignments)?);
                }
            }
            _ => (),
        }
        return Err(PyValueError::new_err(format!(
            "Failed to parse Entrypoint from Expr:\n{expr:#?}"
        )));
    }

    fn get_setup_call<'a>(
        statements: &'a Vec<ast::Stmt>,
        idx: &mut usize,
        assignments: &'a mut BTreeMap<String, ast::Expr>,
    ) -> PyResult<Option<(&'a ast::ExprCall, &'a mut BTreeMap<String, ast::Expr>)>> {
        if *idx < statements.len() {
            match &statements[*idx] {
                ast::Stmt::Assign(assignment) => {
                    assignments.insert_assignments(PyAssignment::Unannotated(assignment))?;
                }
                ast::Stmt::AnnAssign(assignment) => {
                    assignments.insert_assignments(PyAssignment::Annotated(assignment))?;
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
                            return Ok(Some((c, assignments)));
                        }
                    };
                }
                _ => (),
            };
            *idx += 1;
            return Self::get_setup_call(statements, idx, assignments);
        }
        return Ok(None);
    }
}

impl PyStr for ast::Expr {
    fn to_string(&self) -> PyResult<String> {
        if let ast::Expr::Constant(c) = &self {
            if let ast::Constant::Str(s) = &c.value {
                return Ok(s.clone());
            }
        }
        return Err(PyValueError::new_err(format!(
            "Failed to parse String from Expr:\n{self:#?}"
        )));
    }
}

impl PyIdent for ast::Expr {
    fn as_ident(&self) -> PyResult<String> {
        match self {
            ast::Expr::Name(e) => Ok(e.id.to_string()),
            _ => Err(PyTypeError::new_err(format!(
                "Expected Expr::Name in assignment parsing. Found:\n{self:#?}"
            ))),
        }
    }
}

impl PyStrList for ast::Expr {
    fn to_string_vec(&self) -> PyResult<Vec<String>> {
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
        return Err(PyValueError::new_err(format!(
            "Failed to parse Vec<String> from Expr:\n{self:#?}"
        )));
    }
}

impl IdentValueMap for BTreeMap<String, ast::Expr> {
    fn insert_assignments(&mut self, assignment: PyAssignment) -> PyResult<&mut Self> {
        match assignment {
            PyAssignment::Unannotated(assignment) => {
                let mut identifiers = Vec::<String>::new();
                for target in assignment.targets.iter() {
                    identifiers.push(target.as_ident()?);
                }
                for identifier in identifiers {
                    self.insert(identifier, *assignment.value.clone());
                }
            }
            PyAssignment::Annotated(assignment) => {
                let target = &assignment.target;
                if let Some(value) = &assignment.value {
                    self.insert(target.as_ident()?, *value.clone());
                }
            }
        }
        Ok(self)
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
            "{}/tests/inputs/requirements.txt",
            curr_dir.to_str().unwrap()
        );
        let path = Path::new(&path_str);
        let r = RequirementsParser::from_file(&path).unwrap();
        assert_eq!(
            r.requires,
            vec!["flask".to_string(), "pydantic==2.x".to_string()]
        );
    }

    #[test]
    fn make_setuppy() {
        let curr_dir = env::current_dir().unwrap();
        let path_str = format!("{}/tests/inputs/setup.py", curr_dir.to_str().unwrap());
        let path = Path::new(&path_str);
        let s = SetupParser::from_file(&path).unwrap();
        assert_eq!(s.package_name, Some("babelone-test-app".to_string()));
        assert_eq!(s.version, Some("2.0".to_string()));
        assert_eq!(
            s.extra_requires,
            Some(BTreeMap::<String, Vec<Requirement>>::from([
                (
                    "dev".to_string(),
                    vec!["pytest".to_string(), "hypothesis>=6.95.x".to_string()]
                ),
                (
                    "PDF".to_string(),
                    vec!["ReportLab>=1.2".to_string(), "RXP".to_string()]
                )
            ]))
        );
        assert_eq!(s.setup_requires, None);
        assert_eq!(
            s.install_requires,
            Some(vec!["pydantic==2.6.2".to_string(), "fastapi".to_string(),])
        );
        assert_eq!(
            s.entry_points.as_ref().unwrap().console_scripts,
            Some(vec!["hello-world = timmins:hello_world".to_string()])
        );
        assert_eq!(
            s.entry_points.as_ref().unwrap().gui_scripts,
            Some(vec!["hello-world = timmins:hello_world".to_string()])
        );
    }

    #[test]
    fn make_pyproject() {
        let curr_dir = env::current_dir().unwrap();
        let path_str = format!("{}/tests/inputs/pyproject.toml", curr_dir.to_str().unwrap());
        let path = Path::new(&path_str);
        let p = PyProjectParser::from_file(&path).unwrap();
        let build_system = p.build_system.unwrap();
        let project = p.project.unwrap();
        assert_eq!(&build_system.requires, &Some(vec!["hatchling".to_string()]));
        assert_eq!(
            &build_system.build_backend,
            &Some("hatchling.build".to_string())
        );
        assert_eq!(&project.name, &Some("spam-eggs".to_string()));
        assert_eq!(&project.version, &Some("2020.0.0".to_string()));
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
