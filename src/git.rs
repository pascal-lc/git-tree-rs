//! Git integration — enumerate tracked (and optionally untracked) files.

use std::path::PathBuf;
use std::process::Command;

/// Which set of files to list.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileSet {
    /// Only cached (tracked) files: `git ls-files --cached`
    Tracked,
    /// Tracked + untracked, respecting `.gitignore`: adds `--others --exclude-standard`
    Untracked,
    /// Tracked + untracked, ignoring `.gitignore`: adds `--others`
    AllFiles,
}

/// Collect file paths from Git, returning them sorted.
///
/// * `file_set` — which category of files to include.
/// * `target` — optional subdirectory to filter on (relative to repo root).
pub fn list_files(file_set: FileSet, target: Option<&str>) -> Result<Vec<PathBuf>, String> {
    // Locate the repository root
    let root = repo_root()?;

    // Build `git ls-files` arguments
    let mut args: Vec<&str> = vec![
        "-c",
        "core.quotePath=false",
        "ls-files",
        "--cached",
        "--full-name",
    ];

    match file_set {
        FileSet::Tracked => { /* --cached only */ }
        FileSet::Untracked => {
            args.push("--others");
            args.push("--exclude-standard");
        }
        FileSet::AllFiles => {
            args.push("--others");
        }
    }

    // Run git
    let output = Command::new("git")
        .args(&args)
        .current_dir(&root)
        .output()
        .map_err(|e| format!("failed to run git: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git ls-files failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut files: Vec<PathBuf> = stdout
        .lines()
        .map(|l| PathBuf::from(l.trim().to_string()))
        .filter(|p| !p.as_os_str().is_empty())
        .collect();

    // Re-base onto the target directory, if any: keep only entries under it
    // and strip the prefix so tree-building starts from the right root.
    if let Some(tgt) = target {
        let tgt = tgt.trim_end_matches('/');
        files = files
            .into_iter()
            .filter_map(|f| {
                rel_to_target(&f.to_string_lossy(), tgt).map(PathBuf::from)
            })
            .collect();
    }

    // Already sorted by `git ls-files | sort`; but ensure it
    files.sort();

    Ok(files)
}

/// Rebase a repo-root-relative path onto a target directory.
///
/// Returns `Some(rel)` where `rel` is the path with the target prefix
/// removed (or the path itself when it equals `target`), or `None` when the
/// path is outside `target`. Shared by [`list_files`] and
/// [`crate::gitstate::status_map`] to keep target handling consistent.
pub fn rel_to_target(path: &str, target: &str) -> Option<String> {
    let tgt = target.trim_end_matches('/');
    if path == tgt {
        Some(path.to_string())
    } else {
        path.strip_prefix(&format!("{}/", tgt)).map(|rest| rest.to_string())
    }
}

/// Return the absolute path to the repository root.
pub fn repo_root() -> Result<PathBuf, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| format!("failed to run git: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("not inside a Git repository: {}", stderr.trim()));
    }

    let root = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    Ok(PathBuf::from(root))
}

/// Return the repository root directory name (basename).
pub fn repo_name(root: &std::path::Path) -> String {
    root.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(".")
        .to_string()
}
