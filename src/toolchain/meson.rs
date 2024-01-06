use std::collections::HashMap;
use std::path::{Path, PathBuf};

use minijinja::value::Value;
use serde::{Deserialize, Serialize};

use crate::{
    builtin_templates, compute_template, define_license, define_name, error::Result,
    path_validation, BuildTemplate, CreateProject, ProjectOutput, TemplateData,
};

const MESON_FILE: &str = "meson.build";

static MESON_TEMPLATES: &[(&str, &str)] = &builtin_templates!["meson" =>
    ("build.root", "root.build"),
    ("build.cli", "cli.build"),
    ("build.lib", "lib.build"),
    ("build.test", "tests.build"),
    ("source.lib", "lib"),
    ("source.bin", "bin"),
    ("source.test", "test"),
    ("header", "header"),
    ("Dockerfile", "Dockerfile"),
    ("docker.compose", "docker-compose.yml"),
    ("run.tests", "run_tests.sh"),
    ("md.README", "README.md"),
    ("ci.github", "github.yml")
];

/// Kind of a meson project.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ProjectKind {
    /// C-language project
    #[default]
    C,
    /// C++-language project
    Cxx,
}

#[derive(Default)]
/// A meson project data.
pub struct Meson {
    kind: ProjectKind,
}

impl CreateProject for Meson {
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
impl Meson {
    /// Creates a new `Meson` instance.
    pub fn new() -> Self {
        Self {
            kind: ProjectKind::C,
        }
    }

    /// Sets the language
    pub fn kind(mut self, kind: ProjectKind) -> Self {
        self.kind = kind;
        self
    }

    // Build a map Path <-> template
    fn project_structure(
        project_path: &Path,
        name: &str,
        src_ext: &str,
    ) -> (HashMap<PathBuf, &'static str>, Vec<PathBuf>) {
        let name = &name.replace('-', "_");

        let root = project_path.to_path_buf();
        let cli = project_path.join("cli");
        let lib = project_path.join("lib");
        let tests = project_path.join("tests");
        let github = project_path.join(".github/workflows");

        let mut template_files = HashMap::new();

        // All the files in the root of the projects
        template_files.insert(root.join(MESON_FILE), "build.root");
        template_files.insert(root.join("README.md"), "md.README");

        // All the files in the `cli/` directory of the project
        template_files.insert(cli.join(MESON_FILE), "build.cli");
        template_files.insert(cli.join(name).with_extension(src_ext), "source.bin");

        // All the files in the `lib/` directory of the project
        template_files.insert(lib.join(MESON_FILE), "build.lib");
        template_files.insert(lib.join(name).with_extension("h"), "header");
        template_files.insert(lib.join(name).with_extension(src_ext), "source.lib");

        // All the tests for the project, in `tests/`
        template_files.insert(tests.join(MESON_FILE), "build.test");
        template_files.insert(tests.join(name).with_extension(src_ext), "source.test");

        // All docker files
        template_files.insert(root.join("Dockerfile"), "Dockerfile");
        template_files.insert(root.join("docker-compose.yml"), "docker.compose");
        template_files.insert(root.join("run_tests.sh"), "run.tests");

        // Continuous Integration
        template_files.insert(github.join(format!("{name}.yml")), "ci.github");

        (template_files, vec![root, cli, lib, tests, github])
    }
}

impl BuildTemplate for Meson {
    fn define(
        &self,
        project_path: &Path,
        project_name: &str,
        license: &str,
        github_branch: &str,
    ) -> Result<ProjectOutput> {
        let mut context = HashMap::new();
        let (ext, params) = match self.kind {
            ProjectKind::C => ("c", "c_std=c99"),
            ProjectKind::Cxx => ("cpp", "cpp_std=c++11"),
        };

        context.insert("name", Value::from_serializable(&project_name));
        context.insert("branch", Value::from_serializable(&github_branch));
        context.insert("exe", Value::from_serializable(&ext));
        context.insert("params", Value::from_serializable(&params));
        context.insert("license_id", Value::from_serializable(&license));

        let (files, dirs) = Meson::project_structure(project_path, project_name, ext);

        Ok(ProjectOutput {
            context,
            files,
            dirs,
        })
    }

    fn get_templates() -> &'static [(&'static str, &'static str)] {
        MESON_TEMPLATES
    }
}
