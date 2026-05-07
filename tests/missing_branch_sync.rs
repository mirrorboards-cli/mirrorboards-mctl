use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;
use tempfile::TempDir;

#[test]
fn sync_can_create_missing_configured_branch_from_default_branch(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    let origin = root.join("repo-origin");
    init_git_repo(&origin, &[("README.md", "# repo\n")])?;

    let workspace = root.join("workspace");
    fs::create_dir_all(&workspace)?;
    let mirror_toml = format!(
        "[[repositories]]\ngit = \"{}\"\npath = \"repo\"\nbranch = \"graphene-v2\"\n",
        origin.canonicalize()?.display()
    );
    fs::write(workspace.join("mirror.toml"), mirror_toml)?;

    let output = StdCommand::new(env!("CARGO_BIN_EXE_mctl"))
        .current_dir(&workspace)
        .args(["sync", "--create-missing-branches"])
        .output()?;
    assert!(
        output.status.success(),
        "mctl sync failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let repo = workspace.join("repo");
    assert!(repo.join(".git").exists());
    assert_eq!(current_branch(&repo)?, "graphene-v2");
    assert!(remote_branch_exists(&origin, "graphene-v2")?);

    Ok(())
}

#[test]
fn sync_without_create_missing_branches_preserves_missing_branch_failure(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    let origin = root.join("repo-origin");
    init_git_repo(&origin, &[("README.md", "# repo\n")])?;

    let workspace = root.join("workspace");
    fs::create_dir_all(&workspace)?;
    let mirror_toml = format!(
        "[[repositories]]\ngit = \"{}\"\npath = \"repo\"\nbranch = \"graphene-v2\"\n",
        origin.canonicalize()?.display()
    );
    fs::write(workspace.join("mirror.toml"), mirror_toml)?;

    let output = StdCommand::new(env!("CARGO_BIN_EXE_mctl"))
        .current_dir(&workspace)
        .arg("sync")
        .output()?;
    assert!(
        output.status.success(),
        "mctl sync command should report per-repository failures without crashing\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("1 failed"),
        "expected failed summary in stdout, got:\n{}",
        stdout
    );
    assert!(!workspace.join("repo/.git").exists());

    Ok(())
}

fn init_git_repo(path: &Path, files: &[(&str, &str)]) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(path)?;
    run_git(path, &["init", "-b", "main"])?;

    for (relative_path, content) in files {
        let file_path = path.join(relative_path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file_path, content)?;
    }

    run_git(path, &["add", "."])?;
    run_git(
        path,
        &[
            "-c",
            "user.name=Test User",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-m",
            "Initial commit",
        ],
    )?;

    Ok(())
}

fn current_branch(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let output = StdCommand::new("git")
        .args(["branch", "--show-current"])
        .current_dir(path)
        .output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(format!(
            "git branch --show-current failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .into())
    }
}

fn remote_branch_exists(path: &Path, branch: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let output = StdCommand::new("git")
        .args(["rev-parse", "--verify", &format!("refs/heads/{}", branch)])
        .current_dir(path)
        .output()?;
    Ok(output.status.success())
}

fn run_git(path: &Path, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let output = StdCommand::new("git")
        .args(args)
        .current_dir(path)
        .output()?;

    if output.status.success() {
        return Ok(());
    }

    Err(format!(
        "git {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
    .into())
}
