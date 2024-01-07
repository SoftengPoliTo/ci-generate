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

#[derive(Debug)]
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
            license: Cow::Borrowed("MIT"),
            branch: Cow::Borrowed("main"),
            name: Cow::Borrowed(""),
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
        match project_path.file_name().and_then(|x| x.to_str()) {
            Some(name_str) if !name_str.is_empty() && name_str.is_ascii() => Ok(name_str),
            _ => Err(Error::UTF8Check),
        }
    }
}

// Retrieve the license
pub(crate) fn define_license(license: &str) -> Result<&dyn license::License> {
    if license.is_empty() {
        Err(Error::NoLicense)
    } else {
        match license.parse::<&dyn license::License>() {
            Ok(l) => Ok(l),
            Err(_) => Err(Error::InvalidLicense),
        }
    }
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
    use shellexpand::tilde;

    let expanded_path_str = tilde(project_path.to_string_lossy().as_ref()).to_string();
    let project_path: PathBuf = expanded_path_str
        .parse()
        .map_err(|_| Error::WrongExpandUser)?;

    if !project_path.exists() {
        std::fs::create_dir_all(&project_path).map_err(|_| Error::CreationError)?;
    }

    let canonicalized_path =
        std::fs::canonicalize(&project_path).map_err(|_| Error::CanonicalPath)?;
    Ok(canonicalized_path)
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
    use proptest_derive::Arbitrary;
    use tempfile::{tempdir, TempDir};

    static VALID_LICENSES: [&str; 3] = ["MIT", "Apache-2.0", "GPL-3.0"];

    #[derive(Debug, Arbitrary)]
    struct LicenseTest {
        license_str: String,
    }

    proptest! {
        #[test]
        fn define_name_proptest(project_name in "\\PC*", project_path in "\\PC*") {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let temp_path = temp_dir.path().to_path_buf();

            let full_path = temp_path.join(project_path);
            let project_path_str = full_path.file_name().and_then(|x| x.to_str()).unwrap_or("");

            let result = define_name(&project_name, &full_path);

            if project_name.is_empty() || !project_name.is_ascii() {
                if project_path_str.is_empty() || !project_path_str.is_ascii() || project_path_str.contains('/') {
                    prop_assert!(result.is_err());
                } else {
                    prop_assert!(result.is_ok());
                }
            } else {
                prop_assert!(result.is_ok());
            }
        }
    }

    proptest! {
        #[test]
        fn define_license_proptest(data: LicenseTest) {

            let result = define_license(&data.license_str);

            if data.license_str.is_empty() {
                // If the license string is empty, a `NoLicense` error is expected.
                prop_assert!(result.is_err());
            } else if VALID_LICENSES.contains(&&*data.license_str) {
                // If the license string is valid, an Ok result is expected with a License object
                prop_assert!(result.is_ok());
            } else {
                // If the license string is non-empty but invalid, an `InvalidLicense` error is expected.
                prop_assert!(result.is_err());
            }
        }
    }

    #[test]
    fn test_valid_path() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let temp_path = temp_dir.path();
        let valid_path = Path::new("valid_path").join(temp_path);

        let result = path_validation(&valid_path);
        assert!(result.is_ok());
        if let Ok(canonicalized_path) = &result {
            assert!(canonicalized_path.exists());
        }
    }

    #[test]
    fn test_creation_error() {
        let unwritable_path = Path::new("/non_scrivibile");
        let result = path_validation(unwritable_path);
        assert!(result.is_err());
        if let Err(err) = &result {
            assert!(matches!(err, &Error::CreationError));
        }
    }

    #[test]
    fn path_validation_non_ascii_chars() {
        let non_ascii_path = "/path/with/çhåracters";
        let result = path_validation(Path::new(non_ascii_path));

        assert!(result.is_err());
    }

    // Test for path validation for windows
    #[cfg(windows)]
    #[test]
    fn test_valid_path_windows() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let temp_path = temp_dir.path();
        let valid_path = Path::new("valid_path").join(temp_path);

        let result = path_validation_windows(&valid_path);
        assert!(result.is_ok());
        if let Ok(canonicalized_path) = &result {
            assert!(canonicalized_path.exists());
        }
    }

    #[cfg(windows)]
    #[test]
    fn test_creation_error_windows() {
        let unwritable_path = Path::new("C:\\non_scrivibile"); // Cambia il percorso in uno appropriato per Windows
        let result = path_validation_windows(&unwritable_path);
        assert!(result.is_err());
        if let Err(err) = &result {
            assert!(matches!(err, &Error::CreationError));
        }
    }
    #[cfg(windows)]
    #[test]
    fn test_non_ascii_chars_windows() {
        let non_ascii_path = "C:\\path\\with\\çhåracters"; // Cambia il percorso in uno appropriato per Windows
        let result = path_validation_windows(Path::new(non_ascii_path));

        assert!(result.is_err());
    }
}
