use minijinja::value::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    builtin_templates, command::run_command, compute_template, define_license, define_name,
    error::Result, path_validation, BuildTemplate, CreateCi, ProjectOutput, TemplateData,
};

static CARGO_TEMPLATES: &[(&str, &str)] = &builtin_templates!["cargo" =>
    ("md.README", "README.md"),
    ("ci.github", "github.yml"),
    ("ci.github.deploy", "github-deploy.yml"),
    ("ci.github.docker", "github-docker-application.yml"),
    ("docker.amd64", "Dockerfile-amd64"),
    ("docker.arm64", "Dockerfile-arm64"),
    ("rs.proptest", "proptest.rs")
];

/// A cargo project data.
#[derive(Default)]
pub struct Cargo<'a> {
    docker_image_description: Cow<'a, str>,
    ci: bool,
    lib: bool,
}

impl<'a> CreateCi for Cargo<'a> {
    fn create_ci(&self, data: TemplateData) -> Result<()> {
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

impl<'a> Cargo<'a> {
    /// Creates a new `Cargo` instance.
    pub fn new() -> Self {
        Self {
            docker_image_description: std::borrow::Cow::Borrowed("default"),
            lib: false,
            ci: false,
        }
    }
    /// Sets a description
    pub fn docker_image_description(
        mut self,
        docker_image_description: impl Into<Cow<'a, str>>,
    ) -> Self {
        self.docker_image_description = docker_image_description.into();
        self
    }
    /// Sets a library project
    pub fn create_lib(mut self) -> Self {
        self.lib = true;
        self
    }
    /// Sets just ci files
    pub fn only_ci(mut self) -> Self {
        self.ci = true;
        self
    }

    fn project_creation(&self, path: &Path) -> Result<()> {
        if !self.ci {
            if self.lib {
                run_command(path, &["new", "--lib"])?;
            } else {
                run_command(path, &["new"])?;
            }
            run_command(
                &path.join("Cargo.toml"),
                &["add", "--dev", "proptest", "--manifest-path"],
            )?;
        }
        Ok(())
    }

    fn project_structure(
        project_path: &Path,
        name: &str,
        ci: bool,
    ) -> (HashMap<PathBuf, &'static str>, Vec<PathBuf>) {
        let root = project_path.to_path_buf();
        let github = project_path.join(".github/workflows");
        let docker = project_path.join("docker");

        let mut template_files = HashMap::new();

        // README
        template_files.insert(root.join("README.md"), "md.README");

        // Continuous Integration
        template_files.insert(github.join(format!("{name}.yml")), "ci.github");
        template_files.insert(
            github.join(format!("{name}-docker-application.yml")),
            "ci.github.docker",
        );
        template_files.insert(github.join("deploy.yml"), "ci.github.deploy");

        // Docker
        template_files.insert(docker.join("Dockerfile-amd64"), "docker.amd64");
        template_files.insert(docker.join("Dockerfile-arm64"), "docker.arm64");

        if !ci {
            // Proptest
            let tests = project_path.join("project").join("tests");
            template_files.insert(tests.join("proptest.rs"), "rs.proptest");
            (template_files, vec![root, github, docker, tests])
        } else {
            (template_files, vec![root, github, docker])
        }
    }
}

impl<'a> BuildTemplate for Cargo<'a> {
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
        context.insert(
            "docker_image_description",
            Value::from_serializable(&self.docker_image_description),
        );

        fs::remove_dir_all(project_path)?;
        Cargo::project_creation(self, &project_path.join("project"))?;

        let (files, dirs) = Cargo::project_structure(project_path, project_name, self.ci);

        Ok(ProjectOutput {
            files,
            dirs,
            context,
        })
    }

    fn get_templates() -> &'static [(&'static str, &'static str)] {
        CARGO_TEMPLATES
    }
}
