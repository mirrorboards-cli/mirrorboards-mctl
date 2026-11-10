//! Working with a mirror split into include files, using only mctl commands.

use std::fs;
use std::path::Path;
use std::process::{Command as StdCommand, Output};
use tempfile::TempDir;

const ROOT_CONFIG: &str = r#"# Root of the mirror.
[[repositories]]
git = "git@github.com:org/workspace.git"
path = "."

[includes]
paths = [
    "./mirrors/existing.toml",
]
"#;

const EXISTING_INCLUDE: &str = r#"# Libraries shared by every application.

# The library everything else builds on.
[[repositories]]
git = "git@github.com:org/library.git"
path = "group/library"
"#;

#[test]
fn add_with_include_creates_the_file_and_lists_it_in_the_configuration(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();
    write_mirror(root)?;

    let output = mctl(
        root,
        &[
            "add",
            "git@github.com:org/application.git",
            "--path",
            "applications/org/application",
            "--include",
            "./mirrors/applications/org.toml",
        ],
    )?;
    assert_success(&output, "mctl add --include");

    let expected_root = r#"# Root of the mirror.
[[repositories]]
git = "git@github.com:org/workspace.git"
path = "."

[includes]
paths = [
    "./mirrors/existing.toml",
    "./mirrors/applications/org.toml",
]
"#;
    assert_eq!(fs::read_to_string(root.join("mirror.toml"))?, expected_root);

    let expected_include = r#"[[repositories]]
git = "git@github.com:org/application.git"
path = "applications/org/application"
"#;
    assert_eq!(
        fs::read_to_string(root.join("mirrors/applications/org.toml"))?,
        expected_include
    );

    Ok(())
}

#[test]
fn add_to_a_listed_include_keeps_its_comments_and_the_configuration(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();
    write_mirror(root)?;

    // Spelled without "./", the include is still the one already listed.
    let output = mctl(
        root,
        &[
            "add",
            "git@github.com:org/second.git",
            "--path",
            "group/second",
            "--include",
            "mirrors/existing.toml",
        ],
    )?;
    assert_success(&output, "mctl add --include");

    assert_eq!(fs::read_to_string(root.join("mirror.toml"))?, ROOT_CONFIG);

    let expected_include = r#"# Libraries shared by every application.

# The library everything else builds on.
[[repositories]]
git = "git@github.com:org/library.git"
path = "group/library"

[[repositories]]
git = "git@github.com:org/second.git"
path = "group/second"
"#;
    assert_eq!(
        fs::read_to_string(root.join("mirrors/existing.toml"))?,
        expected_include
    );

    Ok(())
}

#[test]
fn add_refuses_a_path_already_mirrored_in_another_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();
    write_mirror(root)?;

    let output = mctl(
        root,
        &[
            "add",
            "git@github.com:org/other.git",
            "--path",
            "group/library",
        ],
    )?;

    assert!(!output.status.success(), "mctl add accepted a taken path");
    assert!(String::from_utf8_lossy(&output.stderr).contains("group/library"));
    assert_eq!(fs::read_to_string(root.join("mirror.toml"))?, ROOT_CONFIG);

    Ok(())
}

#[test]
fn sync_with_path_clones_only_that_repository() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let origins = temp_dir.path().join("origins");
    let root = temp_dir.path().join("mirror");

    let first_origin = origins.join("first");
    let second_origin = origins.join("second");
    init_git_repo(&first_origin)?;
    init_git_repo(&second_origin)?;

    let include = format!(
        "[[repositories]]\ngit = \"{}\"\npath = \"group/first\"\n\n[[repositories]]\ngit = \"{}\"\npath = \"group/second\"\n",
        first_origin.display(),
        second_origin.display()
    );
    fs::create_dir_all(root.join("mirrors"))?;
    fs::write(
        root.join("mirror.toml"),
        "[includes]\npaths = [\"./mirrors/group.toml\"]\n",
    )?;
    fs::write(root.join("mirrors/group.toml"), include)?;

    let output = mctl(&root, &["sync", "--path", "group/first/"])?;
    assert_success(&output, "mctl sync --path");

    assert!(root.join("group/first/.git").exists());
    assert!(!root.join("group/second").exists());

    Ok(())
}

#[test]
fn commands_refuse_an_included_file_as_the_configuration() -> Result<(), Box<dyn std::error::Error>>
{
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();
    write_mirror(root)?;

    let output = mctl(root, &["--config", "mirrors/existing.toml", "sync"])?;

    assert!(
        !output.status.success(),
        "mctl sync ran against an included file"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("is included by"));
    assert!(!root.join("mirrors/group").exists());

    Ok(())
}

#[test]
fn remove_edits_the_include_and_keeps_the_file_description(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();
    write_mirror(root)?;

    let output = mctl(root, &["remove", "group/library"])?;
    assert_success(&output, "mctl remove");

    assert_eq!(fs::read_to_string(root.join("mirror.toml"))?, ROOT_CONFIG);
    assert_eq!(
        fs::read_to_string(root.join("mirrors/existing.toml"))?,
        "# Libraries shared by every application.\n"
    );

    Ok(())
}

#[test]
fn removing_the_first_entry_hands_the_file_description_to_the_next(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();
    write_mirror(root)?;

    let added = mctl(
        root,
        &[
            "add",
            "git@github.com:org/second.git",
            "--path",
            "group/second",
            "--include",
            "./mirrors/existing.toml",
        ],
    )?;
    assert_success(&added, "mctl add --include");

    let removed = mctl(root, &["remove", "group/library"])?;
    assert_success(&removed, "mctl remove");

    let expected_include = r#"# Libraries shared by every application.

[[repositories]]
git = "git@github.com:org/second.git"
path = "group/second"
"#;
    assert_eq!(
        fs::read_to_string(root.join("mirrors/existing.toml"))?,
        expected_include
    );

    Ok(())
}

fn write_mirror(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(root.join("mirrors"))?;
    fs::write(root.join("mirror.toml"), ROOT_CONFIG)?;
    fs::write(root.join("mirrors/existing.toml"), EXISTING_INCLUDE)?;

    Ok(())
}

fn mctl(directory: &Path, args: &[&str]) -> Result<Output, Box<dyn std::error::Error>> {
    let output = StdCommand::new(env!("CARGO_BIN_EXE_mctl"))
        .current_dir(directory)
        .args(args)
        .output()?;

    Ok(output)
}

fn assert_success(output: &Output, command: &str) {
    assert!(
        output.status.success(),
        "{} failed\nstdout:\n{}\nstderr:\n{}",
        command,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn init_git_repo(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(path)?;
    fs::write(path.join("README.md"), "# fixture\n")?;

    run_git(path, &["init", "-b", "main"])?;
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
