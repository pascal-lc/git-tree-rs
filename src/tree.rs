//! Tree data structure, building, and recursive rendering.

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::color::Colors;
use crate::gitstate::GitState;

// ---- data structure --------------------------------------------------------

/// A node in the file tree.
///
/// Both variants carry a [`GitState`]; for directories it is the aggregate
/// (highest-precedence) state among all descendants.
#[derive(Debug)]
pub enum Entry {
    File { state: GitState },
    Dir {
        children: BTreeMap<String, Entry>,
        state: GitState,
    },
}

impl Entry {
    fn dir_mut(&mut self) -> &mut BTreeMap<String, Entry> {
        match self {
            Entry::Dir { children, .. } => children,
            Entry::File { .. } => panic!("tried to get children of a File entry"),
        }
    }

    fn is_dir(&self) -> bool {
        matches!(self, Entry::Dir { .. })
    }

    /// The entry's effective Git state (own state for files, aggregate for
    /// directories).
    pub fn state(&self) -> GitState {
        match self {
            Entry::File { state } => *state,
            Entry::Dir { state, .. } => *state,
        }
    }
}

/// The built tree, ready for rendering.
pub struct Tree {
    pub root_name: String,
    pub root: BTreeMap<String, Entry>,
}

// ---- tree building ---------------------------------------------------------

/// Options controlling tree construction.
#[derive(Default)]
pub struct BuildOptions {
    /// Maximum depth; entries beyond this are pruned. 0 = unlimited.
    pub max_depth: usize,
    /// Show directories only — skip file entries.
    pub dirs_only: bool,
}

/// Build a `Tree` from a sorted list of `(path, state)` pairs.
///
/// Each path is split into components; intermediate components become
/// `Dir` nodes and the final component becomes a `File` carrying its
/// [`GitState`].  Depth pruning happens per-component so that shallow
/// directories are always visible even when their contents are beyond the
/// depth limit.  After insertion, directory nodes are annotated with the
/// highest-precedence state among their descendants.
pub fn build_tree(files: &[(PathBuf, GitState)], opts: &BuildOptions) -> Tree {
    let mut root = BTreeMap::new();

    for (path, state) in files {
        let components: Vec<&str> = path
            .iter()
            .filter_map(|c| c.to_str())
            .collect();

        if components.is_empty() {
            continue;
        }

        let mut current = &mut root;
        for (i, &comp) in components.iter().enumerate() {
            let is_last = i == components.len() - 1;
            let comp_depth = i + 1;

            // Stop walking when we exceed the depth limit — this prunes
            // both deeper directory levels AND the final file component,
            // but shallow directories already added above remain.
            if opts.max_depth > 0 && comp_depth > opts.max_depth {
                break;
            }

            if is_last {
                if opts.dirs_only {
                    // In dirs-only mode, skip leaf files entirely.
                    continue;
                }
                // Insert as File (don't overwrite an existing Dir node),
                // and record its Git state.
                let node = current
                    .entry(comp.to_string())
                    .or_insert(Entry::File { state: GitState::Committed });
                if let Entry::File { state: s } = node {
                    *s = *state;
                }
            } else {
                // Intermediate component — always a directory.
                let node = current
                    .entry(comp.to_string())
                    .or_insert(Entry::Dir {
                        children: BTreeMap::new(),
                        state: GitState::Committed,
                    });
                current = node.dir_mut();
            }
        }
    }

    // Propagate child states up into directory aggregate states.
    for entry in root.values_mut() {
        finalize_state(entry);
    }

    Tree {
        root_name: String::new(),
        root,
    }
}

/// Recursively compute a directory's aggregate [`GitState`] as the
/// highest-ranked state among its descendants; files keep their own state.
fn finalize_state(entry: &mut Entry) -> GitState {
    match entry {
        Entry::File { state } => *state,
        Entry::Dir { children, state } => {
            let agg = children
                .values_mut()
                .map(finalize_state)
                .fold(GitState::Committed, |a, b| {
                    if b.rank() >= a.rank() {
                        b
                    } else {
                        a
                    }
                });
            *state = agg;
            agg
        }
    }
}

// ---- rendering -------------------------------------------------------------

/// Tree-drawing glyphs.
const VT: &str = "│   ";
const EMPTY: &str = "    ";
const BRANCH: &str = "├── ";
const LAST: &str = "└── ";

/// Options controlling rendering.
pub struct RenderOptions<'a> {
    pub colors: &'a Colors,
    pub git_root: &'a Path,
    pub max_depth: usize,
    /// When true, prefix each entry with its Git-state indicator and recolor
    /// regular-file names by state. When false, behavior is the original
    /// type-based coloring with no prefix column.
    pub show_git: bool,
}

/// Render the tree to stdout.
///
/// Returns `Ok(())` on success, or an I/O error.  `BrokenPipe` is
/// silently ignored (the caller should exit 0) so that piping to
/// `head` works as expected.
pub fn render_tree(tree: &Tree, opts: &RenderOptions) -> io::Result<()> {
    let mut out = io::stdout().lock();

    if opts.colors.enabled {
        writeln!(out, "{}{}/\x1b[0m", opts.colors.dir, tree.root_name)?;
    } else {
        writeln!(out, "{}/", tree.root_name)?;
    }

    render_children(&tree.root, "", Path::new(""), opts, 1, &mut out)
}

fn render_children(
    children: &BTreeMap<String, Entry>,
    prefix: &str,
    dir_path: &Path,
    opts: &RenderOptions,
    depth: usize,
    out: &mut impl Write,
) -> io::Result<()> {
    let count = children.len();
    let mut i = 0;

    for (name, entry) in children {
        i += 1;
        let is_last = i == count;
        let is_dir = entry.is_dir();

        let connector = if is_last { LAST } else { BRANCH };
        let child_prefix = if is_last { EMPTY } else { VT };

        let full_prefix = format!("{}{}", prefix, connector);

        let rel_path = dir_path.join(name);
        let fs_path = opts.git_root.join(&rel_path);
        let suffix = if is_dir { "/" } else { "" };

        if opts.show_git {
            let state = entry.state();
            let scolor = opts.colors.git_state_color(state);
            let indicator = state.indicator();
            let ncolor = opts.colors.file_color(&fs_path, is_dir, state, true);
            // {tree}{connector}{reset}{scolor}{indicator}{reset} {ncolor}{name}{suffix}{reset}
            writeln!(
                out,
                "{}{}{}{}{}{} {}{}{}{}",
                opts.colors.tree,
                full_prefix,
                opts.colors.reset,
                scolor,
                indicator,
                opts.colors.reset,
                ncolor,
                name,
                suffix,
                opts.colors.reset,
            )?;
        } else {
            let color =
                opts.colors.file_color(&fs_path, is_dir, GitState::Committed, false);
            if !color.is_empty() {
                writeln!(
                    out,
                    "{}{}{}{}{}{}\x1b[0m",
                    opts.colors.tree,
                    full_prefix,
                    opts.colors.reset,
                    color,
                    name,
                    suffix,
                )?;
            } else {
                writeln!(
                    out,
                    "{}{}{}{}{}",
                    opts.colors.tree,
                    full_prefix,
                    opts.colors.reset,
                    name,
                    suffix,
                )?;
            }
        }

        if is_dir && (opts.max_depth == 0 || depth < opts.max_depth) {
            if let Entry::Dir { children: sub, .. } = entry {
                let new_prefix = format!("{}{}", prefix, child_prefix);
                render_children(sub, &new_prefix, &rel_path, opts, depth + 1, out)?;
            }
        }
    }

    Ok(())
}
