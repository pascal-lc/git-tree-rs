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

    // Filter by target directory
    if let Some(tgt) = target {
        let tgt = tgt.trim_end_matches('/');
        files.retain(|p| {
            let s = p.to_string_lossy();
            s == tgt || s.starts_with(&format!("{}/", tgt))
        });
    }

    // Already sorted by `git ls-files | sort`; but ensure it
    files.sort();

    // Strip the `target` prefix so tree-building starts from the right root
    if let Some(tgt) = target {
        let tgt = tgt.trim_end_matches('/');
        let prefix = format!("{}/", tgt);
        for f in &mut files {
            let s = f.to_string_lossy().into_owned();
            if s == tgt {
                // shouldn't happen for a real file, but be safe
            } else if let Some(rest) = s.strip_prefix(&prefix) {
                *f = PathBuf::from(rest);
            }
        }
    }

    Ok(files)
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
