# git-tree (Rust)

Display Git-tracked files in a tree structure with syntax highlighting —
like the system `tree` command, but scoped to files known to Git.

Written in **Rust** — a single statically-linked binary with zero runtime
dependencies beyond Git itself.

## Quick Start

```bash
# Clone, build, install
git clone git@github.com:chengyaqiang/git-tree-rs.git
cd git-tree-rs
make install

# Ready to use anywhere
git tree -L 2
```

## Makefile Targets

```bash
make help         # show all targets
make build        # cargo build --release
make install      # build + install to PREFIX/bin (default ~/.local/bin)
make uninstall    # remove from PREFIX/bin
make check        # run clippy lints
make test         # run cargo test
make version      # print version number
make clean        # remove build artifacts
```

Custom install path:

```bash
make install PREFIX=/usr/local     # system-wide
make install PREFIX=~/.local       # user-local (default)
```

## Usage

```bash
git tree                  # entire repository (tracked files only)
git tree src/             # subdirectory
git tree -L 2             # depth limit
git tree -d               # directories only
git tree -a               # include untracked files
git tree -A               # include ALL files (.gitignore'd too)
git tree --color=always   # force color output
git tree --color=never    # disable color
git tree --help           # full help
```

## Color Rules

| Entry type     | LS_COLORS key | Default      |
|----------------|---------------|--------------|
| Directory      | `di`          | Bold blue    |
| Executable     | `ex`          | Bold green   |
| Symlink        | `ln`          | Bold cyan    |
| Orphaned link  | `or`          | Red bg       |
| Hidden file    | —             | Dim          |
| Tree connector | —             | Dim          |

Colors automatically follow `LS_COLORS` and respect `NO_COLOR=1`.

## Architecture

```
src/
├── main.rs     clap derive CLI + module wiring
├── color.rs    ANSI codes, LS_COLORS parser, NO_COLOR compliance
├── git.rs      git ls-files integration, file-set selection
└── tree.rs     BTreeMap tree builder, recursive renderer
```

| Module    | Responsibility |
|-----------|---------------|
| `main.rs` | Argument parsing, error handling, orchestration |
| `color.rs`| Color mode detection, LS_COLORS parsing, ANSI escape generation |
| `git.rs`  | `git ls-files` invocation, path filtering, repo root resolution |
| `tree.rs` | In-memory tree from flat path list, depth-limited recursive rendering |

## Requirements

- Rust ≥ 1.70
- Git ≥ 2.0

## Comparison

| Aspect          | Bash (`git-tree`)     | Rust (`git-tree-rs`)  |
|-----------------|-----------------------|------------------------|
| Dependencies    | bash ≥ 4, git         | git only               |
| Portability     | Unix with bash 4      | Cross-platform         |
| Startup         | ~50 ms                | ~2 ms                  |
| Binary size     | 15 KB                 | 768 KB (stripped)      |
| Code            | 436 lines, monolithic | 627 lines, 4 modules   |
| Error handling  | `set -e` + ad-hoc     | `Result<T,E>` + `?`    |

## License

MIT — see [LICENSE](LICENSE).
