//! `mctl pin`: the pins of an image configuration follow the checkouts of a workspace, and only
//! when every checkout is clean and pushed.

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

const OLD_REV: &str = "1111111111111111111111111111111111111111";

/// A workspace with one checkout of a pushed repository and an image configuration that pins
/// it to an older commit. Returns the temporary root and the checkout's HEAD.
fn workspace_with_a_pinned_checkout() -> (TempDir, String) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let origin = root.join("dep.git");
    git(
        root,
        &["init", "--bare", "-b", "main", origin.to_str().unwrap()],
    );

    let checkout = root.join("workspace/deps/dep");
    fs::create_dir_all(&checkout).unwrap();
    git(&checkout, &["init", "-b", "main"]);
    git(&checkout, &["config", "user.email", "test@example.com"]);
    git(&checkout, &["config", "user.name", "Test"]);
    git(
        &checkout,
        &["remote", "add", "origin", origin.to_str().unwrap()],
    );
    fs::write(checkout.join("lib.rs"), "pub fn one() {}\n").unwrap();
    git(&checkout, &["add", "."]);
    git(&checkout, &["commit", "-m", "One"]);
    git(&checkout, &["push", "-u", "origin", "main"]);
    let head = git(&checkout, &["rev-parse", "HEAD"]);

    let images = root.join("package/.github-images");
    fs::create_dir_all(&images).unwrap();
    fs::write(
        images.join("mirror.toml"),
        format!(
            "# Pins of the release.\n\n[[repositories]]\ngit = {:?}\npath = \"deps/dep\"\nrev = \"{OLD_REV}\"\n",
            origin.to_str().unwrap()
        ),
    )
    .unwrap();

    (temp, head)
}

fn pin(root: &Path) -> Output {
    mctl(
        root,
        &[
            "--config",
            "package/.github-images/mirror.toml",
            "pin",
            "--from",
            "workspace",
        ],
    )
}

fn pinned_file(root: &Path) -> String {
    fs::read_to_string(root.join("package/.github-images/mirror.toml")).unwrap()
}

#[test]
fn a_clean_pushed_checkout_moves_the_pin_and_keeps_the_comments() {
    let (temp, head) = workspace_with_a_pinned_checkout();

    let output = pin(temp.path());

    assert!(output.status.success(), "{output:?}");
    let file = pinned_file(temp.path());
    assert!(file.starts_with("# Pins of the release.\n"), "{file}");
    assert!(file.contains(&format!("rev = \"{head}\"")), "{file}");
}

#[test]
fn an_unpushed_or_dirty_checkout_pins_nothing() {
    let (temp, _) = workspace_with_a_pinned_checkout();
    let checkout = temp.path().join("workspace/deps/dep");

    fs::write(checkout.join("lib.rs"), "pub fn two() {}\n").unwrap();
    let dirty = pin(temp.path());

    git(&checkout, &["commit", "-am", "Two"]);
    let unpushed = pin(temp.path());

    for output in [dirty, unpushed] {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "{stderr}");
        assert!(stderr.contains("deps/dep"), "{stderr}");
    }
    assert!(pinned_file(temp.path()).contains(OLD_REV));
}
