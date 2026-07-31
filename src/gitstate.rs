//! Git status detection — classify each tracked file as committed,
//! modified, staged, or untracked by parsing `git status --porcelain`.
//!
//! This module is intentionally separate from [`crate::git`], which handles
//! file-set *enumeration* (`git ls-files`). Here we deal only with the
//! *state* of those files, collapsed from Git's two-field porcelain status
//! into a single value, following the approach used by `eza --git`.

use std::collections::HashMap;
use std::process::Command;

use crate::git;

/// A file's Git state, collapsed from the porcelain `XY` pair.
///
/// Git tracks staged (index vs HEAD) and unstaged (working tree vs index)
/// changes separately; we pick a single representative state with staged
/// taking precedence over modified (e.g. a file that is both staged and
/// further modified is reported as [`Self::Staged`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitState {
    /// Tracked and unmodified — matches HEAD and the working tree.
    Committed,
    /// Has unstaged changes (working tree differs from the index).
    Modified,
    /// Has staged changes (index differs from HEAD).
    Staged,
    /// Not tracked by Git (only shown with `-a` / `-A`).
    Untracked,
}

impl GitState {
    /// Single-character indicator drawn before the filename.
    pub fn indicator(self) -> &'static str {
        match self {
            Self::Committed => "✓",
            Self::Modified => "M",
            Self::Staged => "+",
            Self::Untracked => "?",
        }
    }

    /// Precedence rank used for directory aggregation (higher wins).
    /// Order: staged > modified > untracked > committed.
    pub fn rank(self) -> u8 {
        match self {
            Self::Committed => 0,
            Self::Untracked => 1,
            Self::Modified => 2,
            Self::Staged => 3,
        }
    }
}

/// Collapse a porcelain `XY` pair into a single [`GitState`].
///
/// Returns `None` for ignored entries (`!!`) so the caller can skip them.
/// Staged changes take precedence over unstaged ones.
fn classify_status(x: char, y: char) -> Option<GitState> {
    match (x, y) {
        ('?', '?') => Some(GitState::Untracked),
        ('!', '!') => None,
        (x, _) if x != ' ' && x != '?' && x != '!' => Some(GitState::Staged),
        (_, y) if y != ' ' && y != '?' && y != '!' => Some(GitState::Modified),
        _ => Some(GitState::Committed),
    }
}

/// Build a `path → GitState` map from `git status --porcelain=v1`.
///
/// Keys are returned relative to `target` (when given), matching the output
/// of [`git::list_files`]. Untracked entries (`??`) are included so that
/// `-a` / `-A` mode can annotate them.
pub fn status_map(target: Option<&str>) -> Result<HashMap<String, GitState>, String> {
    let root = git::repo_root()?;

    let output = Command::new("git")
        .args(["-c", "core.quotePath=false", "status", "--porcelain=v1"])
        .current_dir(&root)
        .output()
        .map_err(|e| format!("failed to run git: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git status failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut map: HashMap<String, GitState> = HashMap::new();

    for line in stdout.lines() {
        // Porcelain v1 layout: `XY <path>` — one space separator at index 2.
        let bytes = line.as_bytes();
        if bytes.len() < 3 {
            continue;
        }
        let x = bytes[0] as char;
        let y = bytes[1] as char;
        let raw = line.get(3..).unwrap_or("").trim_end();

        // Rename: `R  old -> new` — annotate the *new* path.
        let path = match raw.rsplit_once(" -> ") {
            Some((_, new)) => new,
            None => raw,
        };

        if path.is_empty() {
            continue;
        }

        let Some(state) = classify_status(x, y) else {
            continue; // ignored (skipped)
        };

        // Re-base onto the target directory, if any.
        let rel = match target {
            Some(t) => match git::rel_to_target(path, t) {
                Some(r) => r,
                None => continue,
            },
            None => path.to_string(),
        };

        map.insert(rel, state);
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_basic() {
        assert_eq!(classify_status(' ', ' '), Some(GitState::Committed));
        assert_eq!(classify_status(' ', 'M'), Some(GitState::Modified));
        assert_eq!(classify_status('M', ' '), Some(GitState::Staged));
        assert_eq!(classify_status('A', ' '), Some(GitState::Staged));
        assert_eq!(classify_status('D', ' '), Some(GitState::Staged));
        assert_eq!(classify_status('?', '?'), Some(GitState::Untracked));
        assert_eq!(classify_status('!', '!'), None);
    }

    #[test]
    fn classify_staged_wins_over_modified() {
        assert_eq!(classify_status('M', 'M'), Some(GitState::Staged));
        assert_eq!(classify_status('A', 'M'), Some(GitState::Staged));
        assert_eq!(classify_status('R', 'M'), Some(GitState::Staged));
    }

    #[test]
    fn rank_ordering() {
        assert!(GitState::Staged.rank() > GitState::Modified.rank());
        assert!(GitState::Modified.rank() > GitState::Untracked.rank());
        assert!(GitState::Untracked.rank() > GitState::Committed.rank());
    }

    #[test]
    fn indicators_are_single_char() {
        for state in [
            GitState::Committed,
            GitState::Modified,
            GitState::Staged,
            GitState::Untracked,
        ] {
            assert_eq!(state.indicator().chars().count(), 1);
        }
    }
}
