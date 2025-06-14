<!-- markdownlint-disable -->
<div align="center">
<h1>Patch Release Me</h1>

[![GitHub](https://img.shields.io/badge/github-%23121011.svg?style=for-the-badge&logo=github&logoColor=white)][github]
[![Crates.io Version](https://img.shields.io/crates/v/patch-release-me?style=for-the-badge)][crates-io]
[![Crates.io Downloads (recent)](https://img.shields.io/crates/dr/patch-release-me?style=for-the-badge)][crates-io]
[![GitHub Stars](https://img.shields.io/github/stars/42ByteLabs/patch-release-me?style=for-the-badge)][github]
[![Licence](https://img.shields.io/github/license/Ileriayo/markdown-badges?style=for-the-badge)][license]

</div>
<!-- markdownlint-restore -->

This is a tool I built to help maintain a number of code bases.

## ✨ Features

- Configuration as Code
  - Define how to patch your code before release
- Versioning Helpers

## 📦 Usage

You can install / use the tool is a number of different ways

### Cargo / Crates.io

```bash
cargo install patch-release-me
```

### GitHub Actions

```yaml
- name: "Patch Release Me"
  uses: 42ByteLabs/patch-release-me@0.6.1
  with:
    # Bump (patch)
    mode: bump
```

### Container Image

**Pull Container from GitHub:**

```bash
docker pull ghcr.io/42bytelabs/patch-release-me:0.6.1
```

**Run Image:***

```bash
docker run -it --rm -v $PWD:/app ghcr.io/42bytelabs/patch-release-me:0.6.1 patch-release-me --help
```

### Manual Install 

```bash
cargo install --git https://github.com/42ByteLabs/patch-release-me
```

## Configuration

```yaml
# Project / Repository Version
version: 1.2.3

#[optional]: name of the software you are releasing
name: "patch-release-me"
#[optional]: repository owner/name
repository: "42ByteLabs/patch-release-me"
#[optional]: Ecosystem to use
ecosystems:
  # Only `Rust` tagged defaults will be used
  - "Rust"
#[optional]: Are the default release locations added
default: true

# Patch Locations
locations:
  # Array of objects
  # Name of the patch
  - name: "Docs Patch"
    paths:
      # Glob supported path to the files you want to patch
      - 'Cargo.toml'
    # [optional]: Exclude dirs/files
    excludes:
      - '/target/'
    # Patterns to use to patch the files
    patterns:
      # Regex Patterns to find what version you want to patch which requires
      # a capture group `(...)`. The patterns are checks are runtime.
      - 'version = "([0-9]\.[0-9]\.[0.9])"'
      # You can also use placeholders
      # {version}, {major}, {minor}, {patch}, {repository}
      - 'version = "{version}"'
```

## 🦸 Support

Please create [GitHub Issues][github-issues] if there are bugs or feature requests.

This project uses [Semantic Versioning (v2)][semver] and with major releases, breaking changes will occur.

## 📓 License

This project is licensed under the terms of the MIT open source license.
Please refer to [MIT][license] for the full terms.

<!-- Resources -->
[license]: ./LICENSE
[semver]: https://semver.org/
[github]: https://github.com/42ByteLabs/patch-release-me
[github-issues]: https://github.com/42ByteLabs/patch-release-me/issues
[crates-io]: https://crates.io/crates/patch-release-me
