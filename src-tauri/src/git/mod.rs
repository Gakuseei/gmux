use git2::{DiffDelta, DiffHunk, DiffLine as Git2DiffLine, DiffOptions, Repository, StatusOptions};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

fn validate_repo_path(workdir: &Path, file: &str) -> Result<PathBuf, String> {
    let canonical_workdir = workdir.canonicalize().map_err(|e| e.to_string())?;
    let full_path = workdir.join(file);
    let canonical_full = full_path.canonicalize().unwrap_or_else(|_| workdir.join(file));
    if !canonical_full.starts_with(&canonical_workdir) {
        return Err("Path escapes repository working directory".to_string());
    }
    Ok(canonical_full)
}

#[derive(serde::Serialize)]
pub struct BranchInfo {
    pub name: String,
    #[serde(rename = "isCurrent")]
    pub is_current: bool,
}

#[derive(serde::Serialize)]
pub struct FileStatus {
    pub path: String,
    pub status: String,
    pub additions: u32,
    pub deletions: u32,
}

#[derive(serde::Serialize, Clone)]
pub struct DiffLineInfo {
    pub origin: String,
    #[serde(rename = "oldLineno")]
    pub old_lineno: Option<u32>,
    #[serde(rename = "newLineno")]
    pub new_lineno: Option<u32>,
    pub content: String,
}

#[derive(serde::Serialize)]
pub struct FileDiff {
    pub path: String,
    pub hunks: Vec<Vec<DiffLineInfo>>,
}

#[tauri::command]
pub fn get_current_branch(path: String) -> Result<Option<String>, String> {
    let repo = Repository::discover(&path).map_err(|e| e.to_string())?;
    let head = repo.head().map_err(|e| e.to_string())?;
    Ok(head.shorthand().map(|s| s.to_string()))
}

#[tauri::command]
pub fn get_branches(path: String) -> Result<Vec<BranchInfo>, String> {
    let repo = Repository::discover(&path).map_err(|e| e.to_string())?;
    let head = repo.head().ok();
    let current_name = head.as_ref().and_then(|h| h.shorthand().map(|s| s.to_string()));

    let branches = repo
        .branches(Some(git2::BranchType::Local))
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for branch in branches {
        let (branch, _) = branch.map_err(|e| e.to_string())?;
        if let Some(name) = branch.name().map_err(|e| e.to_string())? {
            result.push(BranchInfo {
                is_current: current_name.as_deref() == Some(name),
                name: name.to_string(),
            });
        }
    }

    result.sort_by(|a, b| {
        b.is_current.cmp(&a.is_current).then(a.name.cmp(&b.name))
    });

    Ok(result)
}

#[tauri::command]
pub fn switch_branch(path: String, branch: String) -> Result<(), String> {
    let repo = Repository::discover(&path).map_err(|e| e.to_string())?;
    let obj = repo
        .revparse_single(&format!("refs/heads/{}", branch))
        .map_err(|e| e.to_string())?;
    repo.checkout_tree(&obj, None).map_err(|e| e.to_string())?;
    repo.set_head(&format!("refs/heads/{}", branch))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_git_status(path: String) -> Result<Vec<FileStatus>, String> {
    let repo = Repository::discover(&path).map_err(|e| e.to_string())?;
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true);

    let statuses = repo.statuses(Some(&mut opts)).map_err(|e| e.to_string())?;
    let mut result = Vec::new();

    for entry in statuses.iter() {
        let s = entry.status();
        let file_path = entry.path().unwrap_or("").to_string();

        let status_str = if s.contains(git2::Status::WT_NEW) || s.contains(git2::Status::INDEX_NEW) {
            "added"
        } else if s.contains(git2::Status::WT_DELETED) || s.contains(git2::Status::INDEX_DELETED) {
            "deleted"
        } else if s.contains(git2::Status::WT_MODIFIED) || s.contains(git2::Status::INDEX_MODIFIED) {
            "modified"
        } else if s.contains(git2::Status::WT_RENAMED) || s.contains(git2::Status::INDEX_RENAMED) {
            "modified"
        } else {
            continue;
        };

        let (additions, deletions) = count_file_changes(&repo, &file_path, s);

        result.push(FileStatus {
            path: file_path,
            status: status_str.to_string(),
            additions,
            deletions,
        });
    }

    Ok(result)
}

fn count_file_changes(repo: &Repository, file_path: &str, status: git2::Status) -> (u32, u32) {
    let mut adds: u32 = 0;
    let mut dels: u32 = 0;

    if status.contains(git2::Status::WT_NEW) {
        if let Ok(blob) = std::fs::read_to_string(
            repo.workdir()
                .map(|d| d.join(file_path))
                .unwrap_or_default(),
        ) {
            adds = blob.lines().count() as u32;
        }
        return (adds, dels);
    }

    let mut diff_opts = DiffOptions::new();
    diff_opts.pathspec(file_path);

    let diff = if status.intersects(git2::Status::INDEX_NEW | git2::Status::INDEX_MODIFIED | git2::Status::INDEX_DELETED | git2::Status::INDEX_RENAMED) {
        let head_tree = repo.head().ok().and_then(|h| h.peel_to_tree().ok());
        repo.diff_tree_to_index(head_tree.as_ref(), None, Some(&mut diff_opts)).ok()
    } else {
        repo.diff_index_to_workdir(None, Some(&mut diff_opts)).ok()
    };

    if let Some(diff) = diff {
        let _ = diff.foreach(
            &mut |_: DiffDelta, _| true,
            None,
            None,
            Some(&mut |_: DiffDelta, _: Option<DiffHunk>, line: Git2DiffLine| {
                match line.origin() {
                    '+' => adds += 1,
                    '-' => dels += 1,
                    _ => {}
                }
                true
            }),
        );
    }

    (adds, dels)
}

#[tauri::command]
pub fn get_file_diff(path: String, file: String) -> Result<FileDiff, String> {
    let repo = Repository::discover(&path).map_err(|e| e.to_string())?;

    let mut statuses_opts = StatusOptions::new();
    statuses_opts.pathspec(&file);
    let statuses = repo.statuses(Some(&mut statuses_opts)).map_err(|e| e.to_string())?;

    let is_untracked = statuses
        .iter()
        .any(|e| e.status().contains(git2::Status::WT_NEW));

    if is_untracked {
        let workdir = repo.workdir().ok_or("No workdir")?;
        validate_repo_path(workdir, &file)?;
        let content = std::fs::read_to_string(workdir.join(&file)).map_err(|e| e.to_string())?;
        let lines: Vec<DiffLineInfo> = content
            .lines()
            .enumerate()
            .map(|(i, line)| DiffLineInfo {
                origin: "+".to_string(),
                old_lineno: None,
                new_lineno: Some((i + 1) as u32),
                content: line.to_string(),
            })
            .collect();
        return Ok(FileDiff {
            path: file,
            hunks: vec![lines],
        });
    }

    let mut diff_opts = DiffOptions::new();
    diff_opts.pathspec(&file);

    let is_staged = statuses.iter().any(|e| {
        e.status().intersects(
            git2::Status::INDEX_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_DELETED
                | git2::Status::INDEX_RENAMED,
        )
    });

    let diff = if is_staged {
        let head_tree = repo
            .head()
            .ok()
            .and_then(|h| h.peel_to_tree().ok());
        repo.diff_tree_to_index(head_tree.as_ref(), None, Some(&mut diff_opts))
            .map_err(|e| e.to_string())?
    } else {
        repo.diff_index_to_workdir(None, Some(&mut diff_opts))
            .map_err(|e| e.to_string())?
    };

    let hunks: RefCell<Vec<Vec<DiffLineInfo>>> = RefCell::new(Vec::new());
    let current_hunk: RefCell<Vec<DiffLineInfo>> = RefCell::new(Vec::new());

    diff.foreach(
        &mut |_: DiffDelta, _| true,
        None,
        Some(&mut |_: DiffDelta, _: DiffHunk| {
            let mut ch = current_hunk.borrow_mut();
            if !ch.is_empty() {
                hunks.borrow_mut().push(std::mem::take(&mut *ch));
            }
            true
        }),
        Some(&mut |_: DiffDelta, _: Option<DiffHunk>, line: Git2DiffLine| {
            let origin = match line.origin() {
                '+' => "+",
                '-' => "-",
                _ => " ",
            };
            current_hunk.borrow_mut().push(DiffLineInfo {
                origin: origin.to_string(),
                old_lineno: line.old_lineno(),
                new_lineno: line.new_lineno(),
                content: String::from_utf8_lossy(line.content()).to_string(),
            });
            true
        }),
    )
    .map_err(|e| e.to_string())?;

    let remaining = current_hunk.into_inner();
    if !remaining.is_empty() {
        hunks.borrow_mut().push(remaining);
    }

    Ok(FileDiff { path: file, hunks: hunks.into_inner() })
}

#[tauri::command]
pub fn stage_file(path: String, file: String) -> Result<(), String> {
    let repo = Repository::discover(&path).map_err(|e| e.to_string())?;
    let workdir = repo.workdir().ok_or("No workdir")?;
    validate_repo_path(workdir, &file)?;
    let mut index = repo.index().map_err(|e| e.to_string())?;
    index
        .add_path(std::path::Path::new(&file))
        .map_err(|e| e.to_string())?;
    index.write().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn unstage_file(path: String, file: String) -> Result<(), String> {
    let repo = Repository::discover(&path).map_err(|e| e.to_string())?;
    let workdir = repo.workdir().ok_or("No workdir")?;
    validate_repo_path(workdir, &file)?;
    let head = repo.head().map_err(|e| e.to_string())?;
    let tree = head.peel_to_tree().map_err(|e| e.to_string())?;
    let mut index = repo.index().map_err(|e| e.to_string())?;

    match tree.get_path(std::path::Path::new(&file)) {
        Ok(entry) => {
            let obj = repo.find_blob(entry.id()).map_err(|e| e.to_string())?;
            index
                .add_frombuffer(
                    &git2::IndexEntry {
                        ctime: git2::IndexTime::new(0, 0),
                        mtime: git2::IndexTime::new(0, 0),
                        dev: 0,
                        ino: 0,
                        mode: entry.filemode() as u32,
                        uid: 0,
                        gid: 0,
                        file_size: obj.size() as u32,
                        id: entry.id(),
                        flags: 0,
                        flags_extended: 0,
                        path: file.as_bytes().to_vec(),
                    },
                    obj.content(),
                )
                .map_err(|e| e.to_string())?;
        }
        Err(_) => {
            index
                .remove_path(std::path::Path::new(&file))
                .map_err(|e| e.to_string())?;
        }
    }

    index.write().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn revert_file(path: String, file: String) -> Result<(), String> {
    let repo = Repository::discover(&path).map_err(|e| e.to_string())?;
    let workdir = repo.workdir().ok_or("No workdir")?;
    validate_repo_path(workdir, &file)?;
    let full_path = workdir.join(&file);

    let head = repo.head().map_err(|e| e.to_string())?;
    let tree = head.peel_to_tree().map_err(|e| e.to_string())?;
    let obj = tree.get_path(std::path::Path::new(&file));

    match obj {
        Ok(_) => {
            repo.checkout_head(Some(
                git2::build::CheckoutBuilder::new()
                    .force()
                    .path(&file),
            ))
            .map_err(|e| e.to_string())?;
        }
        Err(_) => {
            if full_path.exists() {
                std::fs::remove_file(&full_path).map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}
