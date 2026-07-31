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

## Example

Running `git tree -L 2` in its own repository:

```text
git-tree-rs/
├── .gitignore
├── Cargo.lock
├── Cargo.toml
├── Makefile
├── man/
│   └── git-tree.1
├── README.md
└── src/
    ├── color.rs
    ├── git.rs
    ├── main.rs
    └── tree.rs
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
git tree --no-git-status  # suppress Git status prefixes
git tree --help           # full help
```

## Git Status Prefixes

By default each entry is prefixed with a one-character Git state indicator
(following `eza --git`), so you can spot uncommitted work at a glance:

| State        | Prefix | Color             |
|--------------|--------|-------------------|
| committed    | `✓`    | Dim (tree color)  |
| modified     | `M`    | Yellow (LS_COLORS: `gm`) |
| staged       | `+`    | Green (LS_COLORS: `ga`)  |
| untracked    | `?`    | Red (LS_COLORS: `gu`)    |

Directories show the **aggregate** state of their descendants (staged >
modified > untracked). A file that is both staged and modified is reported
as staged. Regular-file names are recolored by state; directories,
executables and symlinks keep their type color and rely on the prefix.

```text
git-tree-rs/
├── ✓ .gitignore
├── + Cargo.toml          <- staged
├── M src/main.rs         <- modified
└── ✓ src/
    └── + tree.rs
```

Use `--no-git-status` to restore the original prefix-free layout.

## Color Rules

| Entry type     | LS_COLORS key | Default      |
|----------------|---------------|--------------|
| Directory      | `di`          | Bold blue    |
| Executable     | `ex`          | Bold green   |
| Symlink        | `ln`          | Bold cyan    |
| Orphaned link  | `or`          | Red bg       |
| Hidden file    | —             | Dim          |
| Tree connector | —             | Dim          |
| Git: modified  | `gm`          | Yellow bold  |
| Git: staged    | `ga`          | Green bold   |
| Git: untracked | `gu`          | Red bold     |

Colors automatically follow `LS_COLORS` and respect `NO_COLOR=1`.

## Architecture

```
src/
├── main.rs      clap derive CLI + module wiring
├── color.rs     ANSI codes, LS_COLORS parser, NO_COLOR compliance
├── git.rs       git ls-files integration, file-set selection, repo root
├── gitstate.rs  git status --porcelain parsing, GitState classification
└── tree.rs      BTreeMap tree builder, state aggregation, recursive renderer
```

| Module       | Responsibility |
|--------------|---------------|
| `main.rs`    | Argument parsing, error handling, orchestration |
| `color.rs`   | Color mode detection, LS_COLORS parsing, ANSI escape generation |
| `git.rs`     | `git ls-files` invocation, path filtering, repo root resolution |
| `gitstate.rs`| `git status --porcelain` parsing, committed/modified/staged/untracked classification |
| `tree.rs`    | In-memory tree from flat path list, directory state aggregation, depth-limited rendering |

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
