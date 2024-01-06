use std::collections::HashMap;
use std::path::{Path, PathBuf};

use minijinja::value::Value;

use crate::{
    builtin_templates, compute_template, define_license, define_name, error::Result,
    path_validation, BuildTemplate, CreateCi, ProjectOutput, TemplateData,
};

static YARN_TEMPLATES: &[(&str, &str)] = &builtin_templates!["yarn" =>
    ("md.README", "README.md"),
    ("ci.gitlab", ".gitlab-ci.yml"),
    ("ci.github", "github.yml")
];

/// A yarn project data.
#[derive(Default)]
pub struct Yarn;

impl CreateCi for Yarn {
    fn create_ci(&self, data: TemplateData) -> Result<()> {
        let project_path = path_validation(data.project_path)?;
        let project_name = define_name(&data.name, project_path.as_path())?;
        let license = define_license(&data.license)?;
        let template = Yarn.build(
            project_path.as_path(),
            project_name,
            license.id(),
            &data.branch,
        );
        compute_template(template?, license, project_path.as_path())
    }
}

impl Yarn {
    /// Creates a new `Yarn` instance.
    pub fn new() -> Self {
        Self
    }

    fn project_structure(
        project_path: &Path,
        name: &str,
    ) -> (HashMap<PathBuf, &'static str>, Vec<PathBuf>) {
        let root = project_path.to_path_buf();
        let github = project_path.join(".github/workflows");

        let mut template_files = HashMap::new();

        // README
        template_files.insert(root.join("README.md"), "md.README");

        // Continuous Integration
        template_files.insert(root.join(".gitlab-ci.yml"), "ci.gitlab");
        template_files.insert(github.join(format!("{name}.yml")), "ci.github");

        (template_files, vec![root, github])
    }
}

impl BuildTemplate for Yarn {
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
        context.insert("license_id", Value::from_serializable(&license));

        let (files, dirs) = Yarn::project_structure(project_path, project_name);
        Ok(ProjectOutput {
            context,
            files,
            dirs,
        })
    }

    fn get_templates() -> &'static [(&'static str, &'static str)] {
        YARN_TEMPLATES
    }
}
