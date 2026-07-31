//! ANSI color support with LS_COLORS compatibility.
//!
//! Parses the `LS_COLORS` environment variable to match `ls --color` behavior.
//! Respects `NO_COLOR` (https://no-color.org).

use std::collections::HashMap;
use std::env;
use std::io::IsTerminal;

use crate::gitstate::GitState;

/// When to emit ANSI color codes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorMode {
    Always,
    Never,
    Auto,
}

/// Holds ANSI escape codes for each file-type category.
#[derive(Clone)]
pub struct Colors {
    pub dir: String,
    pub exec: String,
    pub symlink: String,
    pub orphan: String,
    pub hidden: String,
    pub tree: String,
    pub reset: String,
    pub enabled: bool,
    pub git_modified: String,
    pub git_staged: String,
    pub git_untracked: String,
}

impl Colors {
    /// Build the color palette from environment and mode.
    pub fn new(mode: ColorMode) -> Self {
        let enabled = match mode {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => {
                // NO_COLOR takes precedence over auto-detection
                if env::var("NO_COLOR").is_ok_and(|v| !v.is_empty()) {
                    false
                } else {
                    std::io::stdout().is_terminal()
                }
            }
        };

        if !enabled {
            return Self {
                dir: String::new(),
                exec: String::new(),
                symlink: String::new(),
                orphan: String::new(),
                hidden: String::new(),
                tree: String::new(),
                reset: String::new(),
                enabled: false,
                git_modified: String::new(),
                git_staged: String::new(),
                git_untracked: String::new(),
            };
        }

        let map = parse_ls_colors();

        Self {
            dir: ansi(&map, "di", "01;34"),
            exec: ansi(&map, "ex", "01;32"),
            symlink: ansi(&map, "ln", "01;36"),
            orphan: ansi(&map, "or", "40;31;01"),
            hidden: ansi_code("02"),
            tree: ansi_code("02"), // dim connectors
            reset: ansi_code("0"),
            enabled: true,
            // Git-state colors follow eza's LS_COLORS keys where they exist
            // (ga=new, gm=modified, gd=deleted); `gu` is a git-tree extension
            // for untracked files. Defaults avoid clashing with dir/exec.
            git_modified: ansi(&map, "gm", "01;33"),
            git_staged: ansi(&map, "ga", "01;32"),
            git_untracked: ansi(&map, "gu", "01;31"),
        }
    }

    /// Pick the color for a [`GitState`] indicator (and regular-file names).
    /// Returns an empty string when colors are disabled. For
    /// [`GitState::Committed`] the indicator reuses the dim tree color so it
    /// reads as quiet scaffolding; committed file *names* keep their type
    /// color (handled in [`Self::file_color`]).
    pub fn git_state_color(&self, state: GitState) -> &str {
        if !self.enabled {
            return "";
        }
        match state {
            GitState::Committed => &self.tree,
            GitState::Modified => &self.git_modified,
            GitState::Staged => &self.git_staged,
            GitState::Untracked => &self.git_untracked,
        }
    }

    /// Pick the right color for a path based on filesystem metadata and Git
    /// state.
    ///
    /// In git mode (`show_git` = true), regular files take their Git-state
    /// color when not committed; directories, executables and symlinks
    /// always keep their type color so the type semantics are preserved.
    /// Out of git mode, behavior is unchanged (type-based coloring only).
    pub fn file_color(
        &self,
        path: &std::path::Path,
        is_dir: bool,
        state: GitState,
        show_git: bool,
    ) -> &str {
        if !self.enabled {
            return "";
        }

        if is_dir {
            return &self.dir;
        }

        // Executables keep their type color.
        if is_executable(path) {
            return &self.exec;
        }

        // Symlinks keep their type color.
        if path.is_symlink() {
            if path.exists() {
                return &self.symlink;
            }
            return &self.orphan;
        }

        // Regular file: a non-committed state overrides the color in git mode.
        if show_git && state != GitState::Committed {
            return self.git_state_color(state);
        }

        // Hidden dotfiles get dimmed.
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if name.starts_with('.') {
            return &self.hidden;
        }

        ""
    }
}

// ---- helpers ---------------------------------------------------------------

/// Parse `LS_COLORS` env var into a `HashMap<String, String>`.
fn parse_ls_colors() -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(raw) = env::var("LS_COLORS") {
        for pair in raw.split(':') {
            if let Some((k, v)) = pair.split_once('=') {
                map.insert(k.to_string(), v.to_string());
            }
        }
    }
    map
}

/// Look up a key in the LS_COLORS map, falling back to `default`.
fn ansi(map: &HashMap<String, String>, key: &str, default: &str) -> String {
    let code = map.get(key).map(|s| s.as_str()).unwrap_or(default);
    format!("\x1b[{}m", code)
}

/// Build an ANSI escape from a raw code string (e.g. "02" → `\x1b[02m`).
fn ansi_code(code: &str) -> String {
    format!("\x1b[{}m", code)
}

/// Check whether a path has any execute bit set (owner/group/other).
#[cfg(unix)]
fn is_executable(path: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(_path: &std::path::Path) -> bool {
    false
}
