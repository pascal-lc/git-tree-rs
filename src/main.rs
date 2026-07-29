//! git-tree — Display Git-tracked files in a tree structure with syntax highlighting.
//!
//! A replacement for the system `tree` command that shows only files
//! known to Git. Installed as `git-tree` on PATH to be used as `git tree`.

mod color;
mod git;
mod tree;

use clap::Parser;
use std::io::{self, Write};
use std::process;

use color::{ColorMode, Colors};
use git::FileSet;
use tree::{BuildOptions, RenderOptions};

// ---- CLI definition ---------------------------------------------------------

/// Display Git-tracked files in a tree structure with syntax highlighting.
#[derive(Parser, Debug)]
#[command(
    name = "git-tree",
    version = env!("CARGO_PKG_VERSION"),
    about = "Display Git-tracked files in a tree structure",
    long_about = "Render the repository's tracked file hierarchy as an ASCII tree, \
                  similar to the system `tree` utility but scoped to files known to Git.",
    after_help = "Color rules (compatible with LS_COLORS):\n  \
                  Directory      Bold blue    (LS_COLORS: di)\n  \
                  Executable     Bold green   (LS_COLORS: ex)\n  \
                  Symlink        Bold cyan    (LS_COLORS: ln)\n  \
                  Orphan link    Red bg       (LS_COLORS: or)\n  \
                  Hidden file    Dim          (.dotfiles)\n  \
                  Tree lines     Dim          (connectors)"
)]
struct Args {
    /// Include untracked files (excluding .gitignore'd)
    #[arg(short = 'a', long = "all")]
    show_all: bool,

    /// Include ALL files, including .gitignore'd
    #[arg(short = 'A', long = "all-files")]
    show_all_files: bool,

    /// Show directories only
    #[arg(short = 'd', long = "dirs-only")]
    dirs_only: bool,

    /// Limit display depth to N levels
    #[arg(short = 'L', value_name = "N")]
    max_depth: Option<usize>,

    /// When to colorize: always, never, auto (default)
    #[arg(
        long = "color",
        value_name = "WHEN",
        default_value = "auto",
        value_parser = ["always", "yes", "force", "never", "no", "none", "auto", "tty"]
    )]
    color_mode: String,

    /// Target directory (default: repository root)
    #[arg(value_name = "DIRECTORY")]
    target: Option<String>,
}

// ---- main -------------------------------------------------------------------

fn main() {
    let args = Args::parse();

    // ---- resolve the Git repository ----
    let repo_root = match git::repo_root() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    };

    // ---- determine file set ----
    let file_set = if args.show_all_files {
        FileSet::AllFiles
    } else if args.show_all {
        FileSet::Untracked
    } else {
        FileSet::Tracked
    };

    // ---- determine color mode ----
    let color_mode = match args.color_mode.as_str() {
        "always" | "yes" | "force" => ColorMode::Always,
        "never" | "no" | "none" => ColorMode::Never,
        _ => ColorMode::Auto,
    };
    let colors = Colors::new(color_mode);

    // ---- collect files from Git ----
    let files = match git::list_files(file_set, args.target.as_deref()) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    };

    if files.is_empty() {
        return;
    }

    // ---- build the tree ----
    let build_opts = BuildOptions {
        max_depth: args.max_depth.unwrap_or(0),
        dirs_only: args.dirs_only,
    };
    let mut tree = tree::build_tree(&files, &build_opts);

    // ---- determine root label ----
    tree.root_name = match &args.target {
        Some(tgt) => {
            let tgt = tgt.trim_end_matches('/');
            std::path::Path::new(tgt)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(tgt)
                .to_string()
        }
        None => git::repo_name(&repo_root),
    };

    // ---- render ----
    let render_opts = RenderOptions {
        colors: &colors,
        git_root: &repo_root,
        max_depth: args.max_depth.unwrap_or(0),
    };
    // Render (ignore broken pipe — e.g. when piping to `head`)
    if let Err(e) = tree::render_tree(&tree, &render_opts) {
        if e.kind() == io::ErrorKind::BrokenPipe {
            // Graceful exit — downstream closed the pipe
            let _ = io::stdout().flush();
            return;
        }
        eprintln!("error: {}", e);
        process::exit(1);
    }
}
