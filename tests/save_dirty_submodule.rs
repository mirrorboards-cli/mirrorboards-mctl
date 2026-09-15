use std::{fs, path::Path, process::Command};
use tempfile::TempDir;

fn git(path: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .current_dir(path)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().to_owned()
}

#[test]
fn save_preserves_dirty_submodules_and_pushes_pending_parent_commits() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    git(root, &["init", "--bare", "origin.git"]);
    git(root, &["init", "-b", "main", "child"]);
    let child = root.join("child");
    git(&child, &["config", "user.email", "test@example.com"]);
    git(&child, &["config", "user.name", "Test"]);
    fs::write(child.join("file"), "initial").unwrap();
    git(&child, &["add", "."]);
    git(&child, &["commit", "-m", "initial"]);
    git(root, &["init", "-b", "main", "parent"]);
    let parent = root.join("parent");
    git(&parent, &["config", "user.email", "test@example.com"]);
    git(&parent, &["config", "user.name", "Test"]);
    git(
        &parent,
        &[
            "remote",
            "add",
            "origin",
            root.join("origin.git").to_str().unwrap(),
        ],
    );
    git(
        &parent,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            child.to_str().unwrap(),
            "child",
        ],
    );
    fs::write(
        parent.join("mirror.toml"),
        format!(
            "[[repositories]]\ngit = {:?}\npath = \".\"\nbranch = \"main\"\n",
            root.join("origin.git").to_str().unwrap()
        ),
    )
    .unwrap();
    git(&parent, &["add", "."]);
    git(&parent, &["commit", "-m", "initial"]);
    git(&parent, &["push", "-u", "origin", "main"]);
    fs::write(parent.join("child/file"), "keep this change").unwrap();
    for pending in [false, true] {
        if pending {
            fs::write(parent.join("parent-file"), "parent change").unwrap();
            git(&parent, &["add", "parent-file"]);
            git(&parent, &["commit", "-m", "pending"]);
        }
        let before = git(&parent, &["rev-parse", "HEAD"]);
        let out = Command::new(env!("CARGO_BIN_EXE_mctl"))
            .current_dir(&parent)
            .args([
                "-c",
                parent.join("mirror.toml").to_str().unwrap(),
                "save",
                "-v",
            ])
            .output()
            .unwrap();
        let stdout = String::from_utf8(out.stdout).unwrap();
        assert!(out.status.success(), "{stdout}");
        assert!(stdout.contains("no stageable changes"), "{stdout}");
        assert!(!stdout.contains("failed"), "{stdout}");
        assert_eq!(before, git(&parent, &["rev-parse", "HEAD"]));
        assert_eq!(
            before,
            git(&root.join("origin.git"), &["rev-parse", "main"])
        );
        assert_eq!(
            fs::read_to_string(parent.join("child/file")).unwrap(),
            "keep this change"
        );
    }
}
