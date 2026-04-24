use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;
use tempfile::TempDir;

#[test]
fn sync_bootstraps_top_level_repo_before_resolving_includes() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();

    let leaf_origin = root.join("leaf-origin");
    init_git_repo(&leaf_origin, &[("README.md", "# leaf\n")])?;

    let mirror_origin = root.join("mirror-origin");
    let include_content = format!(
        "[[repositories]]\ngit = \"{}\"\npath = \"leaf\"\n",
        leaf_origin.canonicalize()?.display()
    );
    init_git_repo(
        &mirror_origin,
        &[("boards/boards.pay2vault.toml", &include_content)],
    )?;

    let workspace = root.join("workspace");
    fs::create_dir_all(&workspace)?;

    let mirror_toml = format!(
        "[[repositories]]\ngit = \"{}\"\npath = \"mirror\"\n\n[includes]\npaths = [\n    \"./mirror/boards/boards.pay2vault.toml\",\n]\n",
        mirror_origin.canonicalize()?.display()
    );
    fs::write(workspace.join("mirror.toml"), mirror_toml)?;

    let mut cmd = StdCommand::new(env!("CARGO_BIN_EXE_mctl"));
    cmd.current_dir(&workspace).arg("sync");
    let output = cmd.output()?;
    assert!(
        output.status.success(),
        "mctl sync failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(workspace.join("mirror/.git").exists());
    assert!(workspace.join("mirror/boards/boards.pay2vault.toml").exists());
    assert!(workspace.join("leaf/.git").exists());

    Ok(())
}

fn init_git_repo(path: &Path, files: &[(&str, &str)]) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(path)?;
    run_git(path, &["init"])?;

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
