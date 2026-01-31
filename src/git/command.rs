//! Git command builder for constructing git CLI commands.

use std::path::Path;
use std::process::Command;

/// Builder for git commands.
#[derive(Debug, Clone)]
pub struct GitCommand {
    /// The git executable path
    git_path: String,
    /// Custom SSH command (GIT_SSH_COMMAND)
    ssh_command: Option<String>,
    /// Working directory for the command
    work_dir: Option<String>,
    /// The git subcommand (e.g., "clone", "status")
    subcommand: String,
    /// Arguments for the subcommand
    args: Vec<String>,
    /// Environment variables
    env_vars: Vec<(String, String)>,
}

impl GitCommand {
    /// Create a new git command builder.
    pub fn new(subcommand: impl Into<String>) -> Self {
        Self {
            git_path: "git".to_string(),
            ssh_command: None,
            work_dir: None,
            subcommand: subcommand.into(),
            args: Vec::new(),
            env_vars: Vec::new(),
        }
    }

    /// Set the git executable path.
    pub fn git_path(mut self, path: impl Into<String>) -> Self {
        self.git_path = path.into();
        self
    }

    /// Set the SSH command.
    pub fn ssh_command(mut self, cmd: impl Into<String>) -> Self {
        self.ssh_command = Some(cmd.into());
        self
    }

    /// Set the working directory.
    pub fn work_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.work_dir = Some(dir.as_ref().to_string_lossy().to_string());
        self
    }

    /// Add an argument.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add multiple arguments.
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(|s| s.into()));
        self
    }

    /// Add an optional argument (only if Some).
    pub fn arg_opt(self, arg: Option<impl Into<String>>) -> Self {
        match arg {
            Some(a) => self.arg(a),
            None => self,
        }
    }

    /// Add a flag argument (e.g., "--verbose").
    pub fn flag(self, flag: impl Into<String>) -> Self {
        self.arg(flag)
    }

    /// Add a flag only if condition is true.
    pub fn flag_if(self, condition: bool, flag: impl Into<String>) -> Self {
        if condition {
            self.flag(flag)
        } else {
            self
        }
    }

    /// Add an option with value (e.g., "--branch", "main").
    pub fn option(self, opt: impl Into<String>, value: impl Into<String>) -> Self {
        self.arg(opt).arg(value)
    }

    /// Add an option only if value is Some.
    pub fn option_opt(
        self,
        opt: impl Into<String>,
        value: Option<impl Into<String>>,
    ) -> Self {
        match value {
            Some(v) => self.option(opt, v),
            None => self,
        }
    }

    /// Add an environment variable.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.push((key.into(), value.into()));
        self
    }

    /// Build the command string (for display/debugging).
    pub fn to_string(&self) -> String {
        let mut parts = vec![self.git_path.clone(), self.subcommand.clone()];
        parts.extend(self.args.clone());

        if let Some(dir) = &self.work_dir {
            format!("(cd {}) {}", dir, parts.join(" "))
        } else {
            parts.join(" ")
        }
    }

    /// Build the std::process::Command.
    pub fn build(&self) -> Command {
        let mut cmd = Command::new(&self.git_path);
        cmd.arg(&self.subcommand);
        cmd.args(&self.args);

        if let Some(dir) = &self.work_dir {
            cmd.current_dir(dir);
        }

        if let Some(ssh_cmd) = &self.ssh_command {
            cmd.env("GIT_SSH_COMMAND", ssh_cmd);
        }

        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        // Disable interactive prompts
        cmd.env("GIT_TERMINAL_PROMPT", "0");

        cmd
    }
}

/// Common git command constructors.
impl GitCommand {
    /// Create a clone command.
    pub fn clone_repo(url: &str, target: &Path) -> Self {
        Self::new("clone")
            .arg(url)
            .arg(target.to_string_lossy().to_string())
    }

    /// Create a clone command with branch.
    pub fn clone_branch(url: &str, target: &Path, branch: &str) -> Self {
        Self::clone_repo(url, target)
            .option("--branch", branch)
            .flag("--single-branch")
    }

    /// Create a shallow clone command.
    pub fn clone_shallow(url: &str, target: &Path, depth: u32) -> Self {
        Self::clone_repo(url, target).option("--depth", depth.to_string())
    }

    /// Create a fetch command.
    pub fn fetch(repo_path: &Path) -> Self {
        Self::new("fetch")
            .work_dir(repo_path)
            .flag("--all")
            .flag("--prune")
    }

    /// Create a pull command.
    pub fn pull(repo_path: &Path) -> Self {
        Self::new("pull").work_dir(repo_path).flag("--ff-only")
    }

    /// Create a push command.
    pub fn push(repo_path: &Path) -> Self {
        Self::new("push").work_dir(repo_path)
    }

    /// Create a status command (porcelain v2).
    pub fn status(repo_path: &Path) -> Self {
        Self::new("status")
            .work_dir(repo_path)
            .flag("--porcelain=v2")
            .flag("--branch")
    }

    /// Create a checkout command.
    pub fn checkout(repo_path: &Path, ref_name: &str) -> Self {
        Self::new("checkout").work_dir(repo_path).arg(ref_name)
    }

    /// Create a checkout command that creates a new branch.
    pub fn checkout_new_branch(repo_path: &Path, branch: &str) -> Self {
        Self::new("checkout")
            .work_dir(repo_path)
            .flag("-b")
            .arg(branch)
    }

    /// Create a rev-parse command to get HEAD.
    pub fn rev_parse_head(repo_path: &Path) -> Self {
        Self::new("rev-parse").work_dir(repo_path).arg("HEAD")
    }

    /// Create a rev-parse command for short HEAD.
    pub fn rev_parse_head_short(repo_path: &Path) -> Self {
        Self::new("rev-parse")
            .work_dir(repo_path)
            .flag("--short")
            .arg("HEAD")
    }

    /// Create a diff command.
    pub fn diff(repo_path: &Path) -> Self {
        Self::new("diff").work_dir(repo_path)
    }

    /// Create a diff --staged command.
    pub fn diff_staged(repo_path: &Path) -> Self {
        Self::new("diff").work_dir(repo_path).flag("--staged")
    }

    /// Create an add command for all files.
    pub fn add_all(repo_path: &Path) -> Self {
        Self::new("add").work_dir(repo_path).flag("-A")
    }

    /// Create a commit command.
    pub fn commit(repo_path: &Path, message: &str) -> Self {
        Self::new("commit")
            .work_dir(repo_path)
            .option("-m", message)
    }

    /// Create a branch command to get current branch.
    pub fn current_branch(repo_path: &Path) -> Self {
        Self::new("rev-parse")
            .work_dir(repo_path)
            .flag("--abbrev-ref")
            .arg("HEAD")
    }

    /// Create a remote get-url command.
    pub fn remote_url(repo_path: &Path, remote: &str) -> Self {
        Self::new("remote")
            .work_dir(repo_path)
            .arg("get-url")
            .arg(remote)
    }

    /// Create a log command.
    pub fn log(repo_path: &Path) -> Self {
        Self::new("log").work_dir(repo_path)
    }

    /// Create a command to check if path is a git repository.
    pub fn is_git_repo(path: &Path) -> Self {
        Self::new("rev-parse")
            .work_dir(path)
            .flag("--is-inside-work-tree")
    }

    /// Create a reset command.
    pub fn reset(repo_path: &Path, mode: &str) -> Self {
        Self::new("reset").work_dir(repo_path).arg(mode)
    }

    /// Create a stash command.
    pub fn stash(repo_path: &Path) -> Self {
        Self::new("stash").work_dir(repo_path)
    }

    /// Create a stash pop command.
    pub fn stash_pop(repo_path: &Path) -> Self {
        Self::new("stash").work_dir(repo_path).arg("pop")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_basic_command() {
        let cmd = GitCommand::new("status").flag("--short");
        assert_eq!(cmd.to_string(), "git status --short");
    }

    #[test]
    fn test_clone_command() {
        let target = PathBuf::from("/tmp/repo");
        let cmd = GitCommand::clone_repo("git@github.com:test/repo.git", &target);
        assert!(cmd.to_string().contains("clone"));
        assert!(cmd.to_string().contains("git@github.com:test/repo.git"));
    }

    #[test]
    fn test_with_work_dir() {
        let repo = PathBuf::from("/home/user/project");
        let cmd = GitCommand::status(&repo);
        assert!(cmd.to_string().contains("/home/user/project"));
    }

    #[test]
    fn test_option() {
        let cmd = GitCommand::new("clone")
            .option("--branch", "develop")
            .arg("url");
        assert!(cmd.to_string().contains("--branch"));
        assert!(cmd.to_string().contains("develop"));
    }

    #[test]
    fn test_conditional_flag() {
        let verbose = true;
        let cmd = GitCommand::new("fetch").flag_if(verbose, "--verbose");
        assert!(cmd.to_string().contains("--verbose"));

        let cmd2 = GitCommand::new("fetch").flag_if(false, "--verbose");
        assert!(!cmd2.to_string().contains("--verbose"));
    }
}
