use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;
use tempfile::TempDir;

#[test]
fn save_pushes_current_branch_and_sets_upstream_when_missing(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    let origin = root.join("origin.git");
    run_git(root, &["init", "--bare", origin.to_str().unwrap()])?;

    let workspace = root.join("workspace");
    fs::create_dir_all(&workspace)?;
    run_git(&workspace, &["init", "-b", "main"])?;
    run_git(&workspace, &["config", "user.email", "test@example.com"])?;
    run_git(&workspace, &["config", "user.name", "Test User"])?;
    run_git(
        &workspace,
        &["remote", "add", "origin", origin.to_str().unwrap()],
    )?;

    let mirror_toml = format!(
        "[[repositories]]\ngit = \"{}\"\npath = \".\"\nbranch = \"main\"\n",
        origin.display()
    );
    fs::write(workspace.join("mirror.toml"), mirror_toml)?;

    // The branch intentionally has no upstream before save, matching a freshly
    // initialized local repository.
    let upstream_before = StdCommand::new("git")
        .current_dir(&workspace)
        .args(["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
        .output()?;
    assert!(!upstream_before.status.success());

    mctl::cli::commands::save::execute("mirror.toml", None, "save test", false, false)?;

    let upstream = git_output(
        &workspace,
        &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
    )?;
    assert_eq!(upstream.trim(), "origin/main");

    let remote_main = git_output(&origin, &["rev-parse", "main"])?;
    let local_main = git_output(&workspace, &["rev-parse", "main"])?;
    assert_eq!(remote_main.trim(), local_main.trim());

    Ok(())
}

fn run_git(path: &Path, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let status = StdCommand::new("git")
        .current_dir(path)
        .args(args)
        .status()?;
    if !status.success() {
        return Err(format!("git {:?} failed", args).into());
    }
    Ok(())
}

fn git_output(path: &Path, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = StdCommand::new("git")
        .current_dir(path)
        .args(args)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(output.stdout)?)
}
