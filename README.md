<h1 align="center">publisher</h1>

<p align="center">
  <a href="https://termapps.zulipchat.com/#narrow/stream/375287-publisher">
    <img alt="Zulip" src="https://img.shields.io/badge/zulip-join_chat-brightgreen.svg?style=flat-square">
  </a>
  <a href="https://crates.io/crates/publisher">
    <img alt="Crates.io" src="https://img.shields.io/crates/v/publisher?style=flat-square">
  </a>
</p>

<p align="center">
  <b>Tool to publish & distribute CLI tools</b>
</p>

<!-- omit from toc -->
## Getting started

1. [Usage](#usage)
2. [Package Repositories](#package-repositories)
3. [Install](#install)
4. [Configuration](#configuration)
5. [Changelog](#changelog)

## Usage

> NOTE: Only supports tools hosted in GitHub for now.

Setup publishing configuration. *(Only needed for first time setup)*.

```
$ publisher init
```

Setup your CI pipeline to build release artifacts. *(Only needed for first time setup)*.

```
$ publisher generate ci
```

Update your code, commit and push to repository with a version tag.

```
$ git tag v1.0.0
$ git push --tags
```

Check that you meet all requirements for publishing to configured [package repositories](#package-repositories).

```
$ publisher check
```

Run the following to publish a version to configured [package repositories](#package-repositories).

```
$ publisher publish 1.0.0
```

Discover more subcommands and options.

```
$ publisher help
```

## Package Repositories

Used for installing the built binary:

- [AUR (binary)](https://aur.archlinux.org)
- [Homebrew](https://homebrew.sh)
- [Scoop](https://scoop.sh)

Used for building from source:

- [AUR](https://aur.archlinux.org)

<!-- publisher install start -->
## Install

`publisher` is available on Linux, macOS & Windows

<!-- omit from toc -->
#### With [Homebrew](https://brew.sh)

```
$ brew install termapps/tap/publisher
```

<!-- omit from toc -->
#### With [AUR](https://aur.archlinux.org)

```
$ yay -S publisher
```

<!-- omit from toc -->
#### With [Scoop](https://scoop.sh)

```
$ scoop bucket add termapps https://github.com/termapps/scoop-bucket
$ scoop install publisher
```

<!-- omit from toc -->
#### With [cargo](https://crates.io/)

```
$ cargo install publisher
```

<!-- omit from toc -->
#### Direct

Pre-built binary executables are available at [releases page](https://github.com/termapps/publisher/releases).

Download, unarchive the binary, and then put the executable in `$PATH`.

<!-- publisher install end -->
## Configuration

Publisher can be configured using `publisher.toml` file. The below options are avaialable:

| Name          |   Type   | Required | Description                                                   |
| ------------- | :------: | :------: | ------------------------------------------------------------- |
| `name`        |  string  | Yes[^1]  | Name of the binary                                            |
| `description` |  string  | Yes[^1]  | Description of the project                                    |
| `homepage`    |  string  | Yes[^1]  | URL of the project homepage                                   |
| `license`     |  string  | Yes[^1]  | License                                                       |
| `repository`  |  string  |   Yes    | URI of the GitHub repository (ex: termapps/publisher)         |
| `exclude`     | string[] |    No    | [Package Repository selection](#package-repository-selection) |
| `aur`         |  object  |    No    | [AUR](#aur)                                                   |
| `aur_bin`     |  object  |    No    | [AUR (binary)](#aur-binary)                                   |
| `homebrew`    |  object  |   Yes    | [Homebrew](#homebrew)                                         |
| `scoop`       |  object  |   Yes    | [Scoop](#scoop)                                               |

[^1]: If `cargo` binary and `Cargo.toml` file are present, they can be omitted from the config.

<!-- omit from toc -->
#### AUR

| Name        |   Type   | Required | Description                             |
| ----------- | :------: | :------: | --------------------------------------- |
| `name`      |  string  |    No    | Name of the package                     |
| `conflicts` | string[] |    No    | Packages in AUR that conflict with this |

- `name` defaults to the binary name.
- Automatically adds `AUR (binary)` package to `conflicts` if it is selected.

<!-- omit from toc -->
#### AUR (binary)

| Name        |   Type   | Required | Description                             |
| ----------- | :------: | :------: | --------------------------------------- |
| `name`      |  string  |    No    | Name of the package                     |
| `conflicts` | string[] |    No    | Packages in AUR that conflict with this |

- `name` defaults to the binary name concatenated with `-bin`.
- Automatically adds `AUR` package to `conflicts` if it is selected.

<!-- omit from toc -->
#### Homebrew

| Name         |   Type   | Required | Description                            |
| ------------ | :------: | :------: | -------------------------------------- |
| `name`       |  string  |    No    | Name of the formula                    |
| `repository` | string[] |   Yes    | GitHub repository for the homebrew tap |

- `name` defaults to the binary name.

<!-- omit from toc -->
#### Scoop

| Name         |   Type   | Required | Description                            |
| ------------ | :------: | :------: | -------------------------------------- |
| `name`       |  string  |    No    | Name of the app                        |
| `repository` | string[] |   Yes    | GitHub repository for the scoop bucket |

- `name` defaults to the binary name.

<!-- omit from toc -->
#### Package Repository selection

- By default, all the available [package repositories](#package-repositories) are selected if not specified in the subcommand.
- If `exclude` is configured, then those will be excluded from the above selected package repositories.

<!-- omit from toc -->
## Contributors
Here is a list of [Contributors](http://github.com/termapps/publisher/contributors)

<!-- omit from toc -->
### TODO

- Package repositories
  + Alpine Linux ([#1](https://github.com/termapps/publisher/issues/1))
  + Nixpkgs ([#10](https://github.com/termapps/publisher/issues/10))
  + Cargo
  + NPM
  + PyPi
- Platforms ([#4](https://github.com/termapps/publisher/issues/4))
- Shell completions ([#8](https://github.com/termapps/publisher/issues/8))
- Manpages ([#9](https://github.com/termapps/publisher/issues/9))
- Maintainer ([#5](https://github.com/termapps/publisher/issues/5))

## Changelog
Please see [CHANGELOG.md](CHANGELOG.md).

<!-- omit from toc -->
## License
MIT/X11

<!-- omit from toc -->
## Bug Reports
Report [here](http://github.com/termapps/publisher/issues).

<!-- omit from toc -->
## Creator
Pavan Kumar Sunkara (pavan.sss1991@gmail.com)

Follow me on [github](https://github.com/users/follow?target=pksunkara), [twitter](http://twitter.com/pksunkara)
