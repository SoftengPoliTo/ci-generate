# sifis-generate

[![Actions Status][actions badge]][actions]
[![LICENSE][license badge]][license]

This tool generates either new projects for some build systems or configuration
files for some Continuous Integration with the use of templates.

Templates define the layout for a project and allow developers to insert data
at runtime.

Each template contains all files necessary to build a project with a build
system, in addition to Continuous Integration and Docker files used to run
tests and implement further checks.

## Supported build systems

| Build system | Languages | Project template | CI style checks | CI build | CI test | CI coverage upload | CI static analysis | CI dynamic analisys | CI license checks |
| - | - | - | - | - | - | - | - | - | - |
| meson | C / C++ | provided | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |:heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| poetry | Python | provided | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :x: | :heavy_check_mark: | :white_check_mark: | :heavy_check_mark: |
| maven | Java | provided | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark:  | :white_check_mark: | :heavy_check_mark: |
| cargo | Rust | offloaded | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |
| yarn | Javascript / Typescript| offloaded | :x: | :heavy_check_mark:  | :x: | :x: | :x: | :white_check_mark: | :heavy_check_mark:  |

:white_check_mark:: Not necessary for the considered language

## Commands

To see the list of supported commands, run: `sifis-generate --help`

Each command has an optional argument to define a license and an optional argument to
 override the project name instead of using the last component of the project-path.
 The default value for the license argument is `MIT`.

### cargo

```
$ sifis-generate cargo [--license LICENSE --name NAME --branch GITHUB_BRANCH] project-path
```

### maven

```
$ sifis-generate maven [--license LICENSE --name NAME --branch GITHUB_BRANCH] project-group project-path
```

### meson

```
$ sifis-generate meson [--kind meson-project-kind] [--license LICENSE --name NAME --branch GITHUB_BRANCH] project-path
```

Admitted values for the `kind` argument:

- `c`
- `c++`

### poetry

```
$ sifis-generate poetry [--license LICENSE --name NAME --branch GITHUB_BRANCH] project-path
```

### yarn

```
$ sifis-generate yarn [--license LICENSE --name NAME --branch GITHUB_BRANCH] project-path
```

## Project Templates

The following templates generate build systems files in addition to the
configuration files for `GitHub` and `GitLab` Continuous Integration.
Some templates also produces files to configure the `Docker` environment.

- **meson-c**
   - `lib` directory for library source files
   - `cli` directory for command line source files
   - `test` directory for tests source files
   - README.md
   - meson.build
   - Dockerfile
   - docker-compose.yml
   - github.yml
   - .gitlab-ci.yml
- **meson-c++**
    - Same files generated by the `meson-c` template but configured for
      the `cpp` language
- **poetry**
   - README.md
   - pyproject.toml
   - .pre-commit-config.yaml
   - github.yml
   - .gitlab-ci.yml
- **maven**
   - `main` directory for library source files
   - `test` directory for tests source files
   - README.md
   - pom.xml
   - github.yml
- **cargo-ci**
   - README.md
   - github-compact.yml
   - github-ubuntu.yml
   - github-macos.yml
   - github-windows.yml
   - github-deploy.yml
   - .gitlab-ci.yml
- **yarn-ci**
   - README.md
   - github.yml
   - .gitlab-ci.yml

## License

Released under the [MIT License](LICENSE).

## Acknowledgements

This software has been developed in the scope of the H2020 project SIFIS-Home with GA n. 952652.

<!-- Links -->
[actions]: https://github.com/sifis-home/sifis-generate/actions
[license]: LICENSES/MIT.txt

<!-- Badges -->
[actions badge]: https://github.com/sifis-home/sifis-generate/workflows/sifis-generate/badge.svg
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg
