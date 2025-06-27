use anyhow::{Result, Context};
use mirror_sdk::{ConfigManager, Repository};
use serde_json;
use tabled::{Tabled, Table};
use super::{Command, print_warning, print_verbose, print_info};

pub struct ListCommand {
    pub json: bool,
}

#[derive(Tabled)]
struct RepositoryTableRow {
    #[tabled(rename = "Hash")]
    hash: String,
    #[tabled(rename = "Git URL")]
    git: String,
    #[tabled(rename = "Path")]
    path: String,
    #[tabled(rename = "Branch")]
    branch: String,
    #[tabled(rename = "Skip Push")]
    skip_push: String,
}

impl Command for ListCommand {
    fn execute(&self, config_manager: &ConfigManager, verbose: bool) -> Result<()> {
        print_verbose("Loading repository configuration", verbose);
        
        if !config_manager.exists() {
            print_warning("Configuration file does not exist. Run 'mctl init' to create one.");
            return Ok(());
        }
        
        let repositories = config_manager.list_repositories()
            .context("Failed to load repositories from configuration")?;
        
        if repositories.is_empty() {
            print_info("No repositories configured.");
            if verbose {
                println!("Add repositories with: mctl add <git-url>");
            }
            return Ok(());
        }
        
        print_verbose(&format!("Found {} repositories", repositories.len()), verbose);
        
        if self.json {
            self.output_json(&repositories)?;
        } else {
            self.output_table(&repositories, verbose)?;
        }
        
        Ok(())
    }
}

impl ListCommand {
    fn output_json(&self, repositories: &[Repository]) -> Result<()> {
        // Create JSON output with hash included
        let json_repos: Vec<serde_json::Value> = repositories.iter()
            .map(|repo| {
                serde_json::json!({
                    "hash": repo.compute_hash(),
                    "git": repo.git,
                    "path": repo.path,
                    "branch": repo.branch,
                    "skip_push": repo.skip_push
                })
            })
            .collect();
        
        let json_output = serde_json::to_string_pretty(&json_repos)
            .context("Failed to serialize repositories to JSON")?;
        
        println!("{}", json_output);
        Ok(())
    }
    
    fn output_table(&self, repositories: &[Repository], verbose: bool) -> Result<()> {
        let table_rows: Vec<RepositoryTableRow> = repositories.iter()
            .map(|repo| RepositoryTableRow {
                hash: repo.compute_hash(),
                git: repo.git.clone(),
                path: repo.path.clone(),
                branch: repo.branch.clone(),
                skip_push: if repo.skip_push { "✓".to_string() } else { "✗".to_string() },
            })
            .collect();
        
        let table = Table::new(table_rows).to_string();
        println!("{}", table);
        
        if verbose {
            println!("\nTotal: {} repositories", repositories.len());
            println!("Use 'mctl show <hash>' for detailed information about a repository");
            println!("Use 'mctl remove <hash>' to remove a repository");
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use mirror_sdk::Repository;

    fn create_test_repository(git: &str, path: &str) -> Repository {
        Repository::new(
            git.to_string(),
            path.to_string(),
            Some("main".to_string()),
            Some(false),
        )
    }

    #[test]
    fn test_list_empty_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Create empty config
        config_manager.create_empty().unwrap();
        
        let list_command = ListCommand { json: false };
        
        // Should not error on empty config
        list_command.execute(&config_manager, false).unwrap();
    }
    
    #[test]
    fn test_list_nonexistent_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        // Delete the temp file so config doesn't exist
        drop(temp_file);
        
        let config_manager = ConfigManager::new(&temp_path);
        let list_command = ListCommand { json: false };
        
        // Should not error on missing config
        list_command.execute(&config_manager, false).unwrap();
    }
    
    #[test]
    fn test_list_with_repositories() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Add test repositories
        let repo1 = create_test_repository("git@github.com:org/repo1.git", "org/repo1");
        let repo2 = create_test_repository("https://github.com/org/repo2.git", "org/repo2");
        
        config_manager.add_repository(repo1).unwrap();
        config_manager.add_repository(repo2).unwrap();
        
        let list_command = ListCommand { json: false };
        list_command.execute(&config_manager, false).unwrap();
    }
    
    #[test]
    fn test_list_json_output() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Add test repository
        let repo = create_test_repository("git@github.com:org/repo.git", "org/repo");
        config_manager.add_repository(repo).unwrap();
        
        let list_command = ListCommand { json: true };
        list_command.execute(&config_manager, false).unwrap();
    }
    
    #[test]
    fn test_json_serialization() {
        let repos = vec![
            create_test_repository("git@github.com:org/repo1.git", "org/repo1"),
            create_test_repository("https://github.com/org/repo2.git", "org/repo2"),
        ];
        
        let list_command = ListCommand { json: true };
        
        // Should not panic on JSON serialization
        list_command.output_json(&repos).unwrap();
    }
}