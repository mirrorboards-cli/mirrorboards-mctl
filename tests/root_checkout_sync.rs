use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;
use tempfile::TempDir;

#[test]
fn sync_supports_repository_checkout_into_root_path() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    let origin = root.join("workspace-origin");
    let mirror_toml = format!(
        "[[repositories]]\ngit = \"{}\"\npath = \".\"\nbranch = \"main\"\n",
        origin.canonicalize().unwrap_or(origin.clone()).display()
    );

    init_git_repo(
        &origin,
        &[
            (".gitignore", "boards/\n"),
            ("Cargo.toml", "[workspace]\n"),
            ("mirror.toml", &mirror_toml),
        ],
    )?;

    let workspace = root.join("workspace");
    fs::create_dir_all(workspace.join("boards"))?;
    fs::write(workspace.join("boards/local-only.txt"), "ignore me\n")?;
    fs::write(workspace.join("mirror.toml"), &mirror_toml)?;

    let output = StdCommand::new(env!("CARGO_BIN_EXE_mctl"))
        .current_dir(&workspace)
        .arg("sync")
        .output()?;
    assert!(
        output.status.success(),
        "mctl sync failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(workspace.join(".git").exists());
    assert!(workspace.join("Cargo.toml").exists());

    let git_status = StdCommand::new("git")
        .arg("status")
        .arg("--short")
        .current_dir(&workspace)
        .output()?;
    assert!(git_status.status.success());
    assert!(
        String::from_utf8_lossy(&git_status.stdout).trim().is_empty(),
        "expected clean git status\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&git_status.stdout),
        String::from_utf8_lossy(&git_status.stderr)
    );

    let status_output = StdCommand::new(env!("CARGO_BIN_EXE_mctl"))
        .current_dir(&workspace)
        .arg("status")
        .arg("--all")
        .output()?;
    assert!(
        status_output.status.success(),
        "mctl status failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&status_output.stdout),
        String::from_utf8_lossy(&status_output.stderr)
    );

    let diff_output = StdCommand::new(env!("CARGO_BIN_EXE_mctl"))
        .current_dir(&workspace)
        .arg("diff")
        .output()?;
    assert!(diff_output.status.success());
    assert!(String::from_utf8_lossy(&diff_output.stdout).contains("No changes to show"));

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

fn run_git(path: &Path, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let output = StdCommand::new("git").args(args).current_dir(path).output()?;

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
