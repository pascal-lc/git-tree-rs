# git-tree (Rust)

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Display Git-tracked files in a tree structure with syntax highlighting —
like the system `tree` command, but scoped to files known to Git.

Written in **Rust** for portability, performance, and maintainability —
a single statically-linked binary with no runtime dependencies.

## Features

- **Git-aware** — by default shows only tracked files (`git ls-files --cached`)
- **Portable** — single binary, no shell interpreter required, works on any Unix
- **Syntax highlighting** — LS_COLORS-compatible coloring for directories,
  executables, symlinks, and hidden files
- **Depth control** — `-L N` limits output depth
- **Pipe-friendly** — auto-detects terminal; handles SIGPIPE gracefully
- **UTF-8 support** — non-ASCII filenames rendered correctly
- **Modular code** — clean module structure: `color`, `git`, `tree`

## Quick Start

```bash
# Build from source
cargo build --release

# Install
cp target/release/git-tree ~/.local/bin/

# Use
git tree -L 2
```

## Usage

```bash
git tree                  # entire repository
git tree src/             # subdirectory only
git tree -L 2             # depth limit
git tree -d               # directories only
git tree -a               # include untracked files
git tree -A               # include ALL files (.gitignore'd too)
git tree --color=always   # force color
git tree --color=never    # disable color
git tree --help           # full help
```

## Architecture

```
src/
├── main.rs    # CLI definition (clap derive), orchestration
├── color.rs   # ANSI color codes, LS_COLORS parsing, NO_COLOR support
├── git.rs     # Git integration (ls-files, repo root detection)
└── tree.rs    # Tree data structure, builder, recursive renderer
```

| Module    | Responsibility |
|-----------|---------------|
| `main.rs` | Argument parsing via `clap` derive, wiring modules, error handling |
| `color.rs`| ANSI escape generation, `LS_COLORS` env var parsing, `NO_COLOR` compliance |
| `git.rs`  | Running `git ls-files`, file-set selection, path filtering |
| `tree.rs` | `Entry` enum (File/Dir), `BTreeMap`-based builder, recursive renderer with depth limiting |

## Requirements

- **Rust** ≥ 1.70 (for `std::io::IsTerminal`)
- **Git** ≥ 2.0

## Development

```bash
cargo build              # debug build
cargo build --release    # release build
cargo test               # run tests
cargo clippy             # lint
```

## Comparison with Bash Version

| Aspect          | Bash (`git-tree`) | Rust (`git-tree-rs`)    |
|-----------------|-------------------|-------------------------|
| Dependencies    | bash ≥ 4, git     | git only                |
| Portability     | Unix with bash 4  | Cross-platform          |
| Performance     | ~50ms (startup)   | ~2ms (startup)          |
| Binary size     | 15 KB (script)    | 768 KB (stripped)       |
| Maintainability | Single 400-line sh| 4 modules, 300 LoC Rust |
| Error handling  | `set -e` / ad-hoc | Typed `Result` + `?`    |
