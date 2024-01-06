pub mod toolchain;
pub use toolchain::*;

pub mod error;
use error::{Error, Result};

mod command;

mod filters;

use minijinja::value::Value;
use minijinja::Environment;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::{create_dir_all, write};
use std::path::{Path, PathBuf};
use tracing::debug;

use filters::*;

static REUSE_TEMPLATE: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/", "dep5"));

pub struct TemplateData<'a> {
    license: Cow<'a, str>,
    branch: Cow<'a, str>,
    name: Cow<'a, str>,
    project_path: &'a Path,
}
impl<'a> TemplateData<'a> {
    /// Creates a new `Common` instance.
    pub fn new(project_path: &'a Path) -> Self {
        Self {
            license: std::borrow::Cow::Borrowed("MIT"),
            branch: std::borrow::Cow::Borrowed("main"),
            name: std::borrow::Cow::Borrowed(""),
            project_path,
        }
    }
    /// Sets a new license.
    pub fn license(mut self, license: impl Into<Cow<'a, str>>) -> Self {
        self.license = license.into();
        self
    }

    /// Sets a new branch.
    pub fn branch(mut self, branch: impl Into<Cow<'a, str>>) -> Self {
        self.branch = branch.into();
        self
    }

    /// Sets a new project_name.
    pub fn name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
}

/// Used to create a CI configuration for a project.
pub trait CreateCi {
    /// Creates a new CI configuration for a project.
    fn create_ci(&self, data: TemplateData) -> Result<()>;
}

/// Used to create a new project.
pub trait CreateProject {
    /// Creates a new project.
    fn create_project(&self, data: TemplateData) -> Result<()>;
}

struct CiTemplate {
    context: HashMap<&'static str, Value>,
    files: HashMap<PathBuf, &'static str>,
    dirs: Vec<PathBuf>,
    env: Environment<'static>,
}

impl CiTemplate {
    fn render(self) -> Result<()> {
        //let mut env = Environment::new();
        let CiTemplate {
            context,
            files,
            dirs,
            mut env,
        } = self;

        // Create dirs
        for dir in dirs {
            debug!("Creating {}", dir.display());
            create_dir_all(dir)?;
        }

        env.add_filter("comment_license", comment_license);
        env.add_filter("hypens_to_underscores", hypens_to_underscores);

        // Fill in templates
        for (path, template_name) in files {
            debug!("Creating {}", path.display());
            let template = env.get_template(template_name)?;
            let filled_template = template.render(&context)?;
            write(path, filled_template)?;
        }
        Ok(())
    }

    fn add_license(&mut self, license: &dyn license::License, project_path: &Path) -> Result<()> {
        let id = license.id();
        let header = license.header();

        // Adds LICENSE directory and license file
        let license_path = project_path.join("LICENSES");
        self.files
            .insert(license_path.join(format!("{}.txt", id)), "build.license");
        self.dirs.push(license_path);

        let text_without_blank: Vec<&str> = license
            .text()
            .lines()
            .skip(2) // Skip a blank line and license id
            .filter(|x| !x.is_empty())
            .collect();

        let mut license_ctx = HashMap::new();

        license_ctx.insert("header", Value::from_serializable(&header));
        license_ctx.insert("text", Value::from_serializable(&text_without_blank));
        license_ctx.insert("id", Value::from_serializable(&id));

        self.context
            .insert("license", Value::from_serializable(&license_ctx));

        self.env.add_template("build.license", license.text())?;

        Ok(())
    }

    fn add_reuse(&mut self, license: &dyn license::License, project_path: &Path) -> Result<()> {
        // Adds .reuse directory and dep5 file
        let reuse_path = project_path.join(".reuse");
        self.files.insert(reuse_path.join("dep5"), "dep5.reuse");
        self.dirs.push(reuse_path);

        // Gets project name and license header
        let name = self.context.get("name");
        let id = license.id();

        let mut reuse = HashMap::new();

        reuse.insert("name", Value::from_serializable(&name));
        reuse.insert("id", Value::from_serializable(&id));

        self.context
            .insert("reuse", Value::from_serializable(&reuse));

        self.env.add_template("dep5.reuse", REUSE_TEMPLATE)?;

        Ok(())
    }
}

struct ProjectOutput {
    files: HashMap<PathBuf, &'static str>,
    dirs: Vec<PathBuf>,
    context: HashMap<&'static str, Value>,
}

/// Build a template
trait BuildTemplate {
    fn define(
        &self,
        project_path: &Path,
        project_name: &str,
        license: &str,
        github_branch: &str,
    ) -> Result<ProjectOutput>;

    fn get_templates() -> &'static [(&'static str, &'static str)];

    fn build(
        &self,
        project_path: &Path,
        project_name: &str,
        license: &str,
        github_branch: &str,
    ) -> Result<CiTemplate> {
        let t = self.define(project_path, project_name, license, github_branch)?;
        let env = build_environment(Self::get_templates());

        Ok(CiTemplate {
            context: t.context,
            files: t.files,
            dirs: t.dirs,
            env,
        })
    }
}

fn build_environment(templates: &'static [(&'static str, &'static str)]) -> Environment<'static> {
    let mut environment = Environment::new();
    for (name, src) in templates {
        environment
            .add_template(name, src)
            .expect("Internal error, built-in template");
    }

    environment
}
// Retrieve the project name
pub(crate) fn define_name<'a>(project_name: &'a str, project_path: &'a Path) -> Result<&'a str> {
    if !project_name.is_empty() && project_name.is_ascii() {
        Ok(project_name)
    } else {
        let name = match project_path.file_name().and_then(|x| x.to_str()) {
            Some(x) => Ok(x),
            None => Err(Error::UTF8Check),
        };
        name
    }
}
// Retrieve the license
pub(crate) fn define_license(license: &str) -> Result<&dyn license::License> {
    let license = license
        .parse::<&dyn license::License>()
        .map_err(|_| Error::NoLicense)?;
    Ok(license)
}
// Compute template
pub(crate) fn compute_template(
    mut template: CiTemplate,
    license: &dyn license::License,
    project_path: &Path,
) -> Result<()> {
    template.add_reuse(license, project_path)?;
    template.add_license(license, project_path)?;
    template.render()
}

// Performs a path validation for unix/macOs
#[cfg(not(windows))]
pub fn path_validation(project_path: &Path) -> Result<PathBuf> {
    use expanduser::expanduser;
    use std::fs;
    let project_path = if project_path.starts_with("~") {
        let project_path = match expanduser(project_path.display().to_string()) {
            Ok(p) => p,
            Err(_) => return Err(Error::WrongExpandUser),
        };
        project_path
    } else {
        project_path.to_path_buf()
    };

    if !project_path.try_exists()? {
        fs::create_dir(&project_path)?;
    }
    let project_path = std::fs::canonicalize(project_path);
    match project_path {
        Ok(x) => Ok(x),
        _ => Err(Error::CanonicalPath),
    }
}
// Performs a path validation for Windows
#[cfg(windows)]
pub fn path_validation(project_path: &Path) -> Result<PathBuf> {
    use homedir::get_my_home;
    // Creation of the $HOME directory
    let home = get_my_home();
    let mut home = match home {
        Ok(x) => match x {
            Some(h) => h,
            None => return Err(Error::HomeDir),
        },
        _ => return Err(Error::HomeDir),
    };
    // Path validation
    let mut project_path = if project_path.starts_with(r#"~\"#) {
        let str = match project_path.to_str() {
            Some(s) => s,
            None => return Err(Error::WrongExpandUser),
        };
        let str = str.replace("~\\", "");
        home.push(Path::new(&str));
        home
    } else {
        project_path.to_path_buf()
    };
    // extenduser in case of relative path
    project_path = if project_path.is_relative() {
        let absolute_path = match std::fs::canonicalize(project_path) {
            Ok(ap) => ap,
            Err(_) => return Err(Error::CanonicalPath),
        };
        absolute_path
    } else {
        project_path
    };

    let str = match project_path.to_str() {
        Some(s) => {
            s.replace(r#"\\?\"#, "");
            Ok(Path::new(&s).to_path_buf())
        }
        None => return Err(Error::UTF8Check),
    };
    str
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Test for input strings to define_name()
    proptest! {
        #[test]
        fn test_diff_alphabet(s in "[\\p{Greek}][\\p{Cyrillic}]*{0,}"){
            prop_assert!(define_name(&s, Path::new("~/Desktop")).is_ok());
        }
        #[test]
        fn test_valid_word(s in "[[:word:]]*{0,}"){
            prop_assert!(define_name(&s, Path::new("~/Desktop")).is_ok());
        }
        #[test]
        fn test_emoji(s in "[\\p{Emoji}]*{0,}"){
            prop_assert!(define_name(&s, Path::new("~/Desktop")).is_ok());
        }
    }

    // Test for input strings to define_license()
    #[test]
    fn define_license_valid_test() {
        assert!(define_license("AFL-3.0").is_ok())
    }
    #[test]
    fn define_license_invalid_test() {
        assert!(define_license("POL-3.0").is_err());
    }

    // Test for path validation for unix
    #[test]
    fn path_validation_0() {
        assert!(path_validation(Path::new("")).is_err())
    }
    #[test]
    fn path_validation_empty() {
        assert!(path_validation(Path::new("")).is_err());
    }
    #[test]
    fn path_validation_invalid() {
        assert!(
            path_validation(Path::new("~//Desktop/GitHub/ci-generate/../../ci-generate")).is_err()
        );
    }
    #[test]
    fn path_validation_valid() {
        assert!(path_validation(Path::new("tests/common/mod.rs")).is_ok());
    }

    // Test for path validation for windows
    #[cfg(windows)]
    #[test]
    fn path_validation_empty() {
        assert!(path_validation(Path::new("")).is_err());
    }
    #[cfg(windows)]
    #[test]
    fn path_validation_invalid() {
        assert!(path_validation(Path::new("~\\C:\\Users\\..\\..\\Documents")).is_err());
    }
    #[cfg(windows)]
    #[test]
    fn path_validation_valid() {
        assert!(path_validation(Path::new("~\\")).is_ok());
    }
}
