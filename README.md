<!--
SPDX-FileCopyrightText: 2025 Chen Linxuan <me@black-desk.cn>

SPDX-License-Identifier: MIT
-->

# git-sort

[![checks][badge-shields-io-checks]][actions]
[![commit activity][badge-shields-io-commit-activity]][commits]
[![contributors][badge-shields-io-contributors]][contributors]
[![release date][badge-shields-io-release-date]][releases]
![commits since release][badge-shields-io-commits-since-release]
[![codecov][badge-shields-io-codecov]][codecov]

[badge-shields-io-checks]:
  https://img.shields.io/github/check-runs/black-desk/git-sort/master

[actions]: https://github.com/black-desk/git-sort/actions

[badge-shields-io-commit-activity]:
  https://img.shields.io/github/commit-activity/w/black-desk/git-sort/master

[commits]: https://github.com/black-desk/git-sort/commits/master

[badge-shields-io-contributors]:
  https://img.shields.io/github/contributors/black-desk/git-sort

[contributors]: https://github.com/black-desk/git-sort/graphs/contributors

[badge-shields-io-release-date]:
  https://img.shields.io/github/release-date/black-desk/git-sort

[releases]: https://github.com/black-desk/git-sort/releases

[badge-shields-io-commits-since-release]:
  https://img.shields.io/github/commits-since/black-desk/git-sort/latest

[badge-shields-io-codecov]:
  https://codecov.io/github/black-desk/git-sort/graph/badge.svg?token=6TSVGQ4L9X
[codecov]: https://codecov.io/github/black-desk/git-sort

en | [zh_CN](README.zh_CN.md)

> [!WARNING]
>
> This English README is translated from the Chinese version using LLM and may
> contain errors.

A git extension command that sorts commits by their topological order on a
reference branch. Useful for backport workflows where commits need to be
applied in the correct dependency order.

## Usage

```
git-sort [OPTIONS] [INPUT]

ARGS:
    <INPUT>    Input file containing commit hashes (one per line).
               Use '-' for stdin. [default: -]

OPTIONS:
    -o, --output <FILE>    Output file. Use '-' for stdout. [default: -]
        --reference <REF>  Reference branch for topological ordering [default: HEAD]
    -h, --help             Print help
    -V, --version          Print version
```

### Input Format

Each line contains a full commit hash, optionally followed by a tab and the
commit title:

    <full-commit-hash>\t<optional-title>

The output preserves the input format, only changing the order.

### Examples

```bash
# Sort commits from a file
git-sort commits.txt

# Sort commits from stdin
git log --format='%H\t%s' main..feature | git-sort

# Use a specific reference branch
git-sort --reference main commits.txt -o sorted.txt

# Can also be invoked as a git subcommand if in PATH
git sort commits.txt
```

### Performance

For large repositories, it is recommended to generate a commit-graph first
for better performance:

```bash
git commit-graph write
```

If no commit-graph is found, git-sort will print a warning.

## License

Unless otherwise specified, the code of this project are open source under the
GNU General Public License version 3 or any later version, while documentation,
configuration files, and scripts used in the development and maintenance process
are open source under the MIT License.

This project complies with the [REUSE specification].

You can use [reuse-tool](https://github.com/fsfe/reuse-tool) to generate the
SPDX list for this project:

```bash
reuse spdx
```

[REUSE specification]: https://reuse.software/spec-3.3/
