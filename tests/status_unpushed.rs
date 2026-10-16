use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

fn git(path: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(path)
        .args(args)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn mctl(path: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mctl"))
        .current_dir(path)
        .args(args)
        .output()
        .unwrap()
}

fn assert_status(path: &Path, expected: &str, synced: bool) {
    for args in [vec!["status"], vec!["status", "--detailed"]] {
        let output = mctl(path, &args);
        let stdout = String::from_utf8(output.stdout).unwrap();

        assert!(output.status.success(), "{stdout}");
        assert!(stdout.contains(expected), "{stdout}");
        assert_eq!(stdout.contains("All repositories are synced"), synced);
    }
}

#[test]
fn failed_initial_push_stays_visible_and_save_retries_without_new_commit() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    git(root, &["init", "-b", "main", "workspace"]);

    let workspace = root.join("workspace");
    let origin = root.join("origin.git");
    git(&workspace, &["config", "user.email", "test@example.com"]);
    git(&workspace, &["config", "user.name", "Test"]);
    git(
        &workspace,
        &["remote", "add", "origin", origin.to_str().unwrap()],
    );
    fs::write(
        workspace.join("mirror.toml"),
        format!(
            "[[repositories]]\ngit = {:?}\npath = \".\"\n",
            origin.to_str().unwrap()
        ),
    )
    .unwrap();

    let failed_save = mctl(&workspace, &["save"]);
    let stdout = String::from_utf8(failed_save.stdout).unwrap();

    assert!(!failed_save.status.success(), "{stdout}");
    assert!(stdout.contains("committed but push failed"), "{stdout}");
    assert!(stdout.contains("1 failed"), "{stdout}");
    assert!(!stdout.contains("1 saved"), "{stdout}");
    assert_eq!(git(&workspace, &["status", "--porcelain"]), "");
    assert_status(&workspace, "No upstream", false);

    let committed_head = git(&workspace, &["rev-parse", "HEAD"]);
    git(root, &["init", "--bare", origin.to_str().unwrap()]);

    let retry = mctl(&workspace, &["save"]);
    assert!(retry.status.success(), "{retry:?}");
    assert_eq!(committed_head, git(&workspace, &["rev-parse", "HEAD"]));
    assert_eq!(committed_head, git(&origin, &["rev-parse", "main"]));
    assert_status(&workspace, "All repositories are synced", true);

    git(&workspace, &["commit", "--allow-empty", "-m", "pending"]);
    assert_status(&workspace, "main", false);

    git(&workspace, &["push"]);
    git(&workspace, &["reset", "--hard", "HEAD~1"]);
    assert_status(&workspace, "main", false);

    let behind_save = mctl(&workspace, &["save"]);
    let stdout = String::from_utf8(behind_save.stdout).unwrap();
    assert!(behind_save.status.success(), "{stdout}");
    assert!(stdout.contains("1 skipped"), "{stdout}");

    git(&workspace, &["checkout", "--detach"]);
    assert_status(&workspace, "detached", false);
}
