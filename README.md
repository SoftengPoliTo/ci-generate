# sifis-generate

This tool generates either a new project for some build systems or configuration
files for some Continuous Integration with the use of templates.

Templates define the layout for a project and allow developers to insert data
at runtime.

Each template contains all files necessary to build a project with a build
system, in addition to Continuous Integration and Docker files used to run
tests and implement further checks.

## Supported build systems

- [x] meson
- [x] setuptools
- [x] maven

## Build systems CI files

- [x] cargo
- [x] yarn

## Commands

### new

```
$ sifis-generate new --template project-template project-name
```

## Project Templates

The following templates generate build systems files in addition to the
configuration files for `GitHub` and `GitLab` Continuous Integration.
Some templates also produces files to configure the `Docker` environment.

- **meson-c**
   - `lib` directory for library source files
   - `cli` directory for command line source files
   - `test` directory for tests source files
   - meson.build
   - Dockerfile
   - docker-compose.yml
   - github.yml
   - .gitlab-ci.yml
- **meson-c++**
    - Same files generated by the `meson-c` template but configured for
      the `cpp` language
- **setuptools**
   - setup.py
   - setup.cfg
   - pyproject.toml
   - .pre-commit-config.yaml
   - github.yml
   - .gitlab-ci.yml
- **maven**
   - `main` directory for library source files
   - `test` directory for tests source files
   - pom.xml
   - github.yml
- **cargo**
   - github.yml
   - .gitlab-ci.yml
- **yarn**
   - github.yml
   - .gitlab-ci.yml

## Acknowledgements

This software has been developed in the scope of the H2020 project SIFIS-Home with GA n. 952652.
