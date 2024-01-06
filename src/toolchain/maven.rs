use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use minijinja::value::Value;

use crate::{
    builtin_templates, compute_template, define_license, define_name, error::Result,
    path_validation, BuildTemplate, CreateProject, ProjectOutput, TemplateData,
};

static MAVEN_TEMPLATES: &[(&str, &str)] = &builtin_templates!["maven" =>
    ("java.entry", "Entry.java"),
    ("java.example", "Example.java"),
    ("xml.pom", "pom.xml"),
    ("xml.checkstyle", "checkstyle.xml"),
    ("md.README", "README.md"),
    ("ci.github", "github.yml")
];

const MAIN: &str = "main/java";
const TESTS: &str = "test/java";

#[derive(Default)]
/// A maven project.
pub struct Maven<'a> {
    group: Cow<'a, str>,
}

impl<'a> CreateProject for Maven<'a> {
    fn create_project(&self, data: TemplateData) -> Result<()> {
        let project_path = path_validation(data.project_path)?;
        let project_name = define_name(&data.name, project_path.as_path())?;
        let license = define_license(&data.license)?;
        let template = self.build(
            project_path.as_path(),
            project_name,
            license.id(),
            &data.branch,
        );
        compute_template(template?, license, project_path.as_path())
    }
}
impl<'a> Maven<'a> {
    /// Creates a new `Maven` instance.
    pub fn new() -> Self {
        Self { group: std::borrow::Cow::Borrowed("group") }
    }
    /// Sets a group
    pub fn group(mut self, group: impl Into<Cow<'a, str>>) -> Self {
        self.group = group.into();
        self
    }

    fn project_structure(
        project_path: &Path,
        group: &str,
        name: &str,
    ) -> (HashMap<PathBuf, &'static str>, Vec<PathBuf>) {
        let root = project_path.to_path_buf();
        let main = project_path.join(format!("src/{MAIN}/{group}/{name}"));
        let tests = project_path.join(format!("src/{TESTS}/{group}/{name}/example"));
        let github = project_path.join(".github/workflows");

        let mut template_files = HashMap::new();

        // All the files in the root of the projects
        template_files.insert(root.join("pom.xml"), "xml.pom");
        template_files.insert(root.join("checkstyle.xml"), "xml.checkstyle");
        template_files.insert(root.join("README.md"), "md.README");
        template_files.insert(root.join("LICENSE.md"), "build.license");

        // All files in the main directory
        template_files.insert(main.join("Entry.java"), "java.entry");

        // All files in the test directory
        template_files.insert(tests.join("Example.java"), "java.example");

        // Continuous integration files
        template_files.insert(github.join(format!("{name}.yml")), "ci.github");

        (template_files, vec![root, main, tests, github])
    }
}

impl<'a> BuildTemplate for Maven<'a> {
    fn define(
        &self,
        project_path: &Path,
        project_name: &str,
        license: &str,
        github_branch: &str,
    ) -> Result<ProjectOutput> {
        let mut context = HashMap::new();

        context.insert("name", Value::from_serializable(&project_name));
        context.insert("branch", Value::from_serializable(&github_branch));
        context.insert("group", Value::from_serializable(&self.group));
        context.insert("license_id", Value::from_serializable(&license));

        let (files, dirs) = Maven::project_structure(project_path, &self.group, project_name);

        Ok(ProjectOutput {
            files,
            dirs,
            context,
        })
    }

    fn get_templates() -> &'static [(&'static str, &'static str)] {
        MAVEN_TEMPLATES
    }
}
