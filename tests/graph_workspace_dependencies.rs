//! `mctl graph`: a crate's dependency inherited from its workspace (`name = { workspace = true }`)
//! takes its path from the workspace root's `[workspace.dependencies]`, and the repository of
//! that path belongs to the closure.

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

#[test]
fn a_dependency_inherited_from_the_workspace_joins_the_closure() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    write(
        &root.join("mirror.toml"),
        r#"
[[repositories]]
git = "git@example.com:org/app.git"
path = "org/app"

[[repositories]]
git = "git@example.com:org/sdk.git"
path = "org/sdk"

[[repositories]]
git = "git@example.com:org/chain.git"
path = "org/chain"
"#,
    );
    write(
        &root.join("org/app/Cargo.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\n\n[workspace]\n\n[dependencies]\nsdk = { path = \"../sdk/crates/sdk\" }\n",
    );
    write(
        &root.join("org/sdk/Cargo.toml"),
        "[workspace]\nmembers = [\"crates/sdk\"]\n\n[workspace.dependencies]\nchain = { path = \"../chain\" }\n",
    );
    write(
        &root.join("org/sdk/crates/sdk/Cargo.toml"),
        "[package]\nname = \"sdk\"\nversion = \"0.1.0\"\n\n[dependencies]\nchain = { workspace = true }\n",
    );
    write(
        &root.join("org/chain/Cargo.toml"),
        "[package]\nname = \"chain\"\nversion = \"0.1.0\"\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_mctl"))
        .current_dir(root)
        .args(["graph", "org/app", "--format", "json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let closure: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let repos = closure["repos"].as_object().unwrap();
    assert!(repos.contains_key("org/chain"), "{closure}");
    assert!(repos.contains_key("org/sdk"), "{closure}");
}
