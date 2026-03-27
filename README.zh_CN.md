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

[English](README.md) | zh_CN

一个 git 扩展命令，按照参考分支上的拓扑序对 commits 进行排序。适用于 backport 工作流中需要按正确的依赖顺序应用 commits 的场景。

## 使用

```
git-sort [OPTIONS] [INPUT]

参数:
    <INPUT>    包含 commit 哈希的输入文件（每行一个）。
               使用 '-' 表示标准输入。 [默认: -]

选项:
    -o, --output <FILE>    输出文件。使用 '-' 表示标准输出。 [默认: -]
        --reference <REF>  用于拓扑排序的参考分支 [默认: HEAD]
    -h, --help             显示帮助
    -V, --version          显示版本
```

### 输入格式

每行包含完整的 commit 哈希，后面可选地跟着制表符和 commit 标题：

    <完整commit哈希>\t<可选标题>

输出保持输入格式，仅改变顺序。

### 示例

```bash
# 从文件排序 commits
git-sort commits.txt

# 从标准输入排序 commits
git log --format='%H\t%s' main..feature | git-sort

# 使用特定的参考分支
git-sort --reference main commits.txt -o sorted.txt

# 如果在 PATH 中，也可以作为 git 子命令调用
git sort commits.txt
```

### 性能

对于大型仓库，建议先生成 commit-graph 以获得更好的性能：

```bash
git commit-graph write
```

如果没有找到 commit-graph，git-sort 会打印警告。

## 许可证

除非另有说明，本项目的代码以 GNU 通用公共许可证第三版或任何更新版本开源，
而文档、配置文件以及开发和维护过程中使用的脚本以 MIT 许可证开源。

本项目遵守 [REUSE规范]。

你可以使用 [reuse-tool](https://github.com/fsfe/reuse-tool) 生成这个项目的 SPDX 列表：

```bash
reuse spdx
```

[REUSE规范]: https://reuse.software/spec-3.3/
