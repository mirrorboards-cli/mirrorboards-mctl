//! `mctl graph` — domknięcie zależności katalogu, policzone z prawdziwych
//! plików workspace'u.
//!
//! W odróżnieniu od reszty komend KOŃCZY SIĘ NIEZEROWO. Martwa ścieżka
//! w manifeście to jest właśnie to, co ta komenda ma znajdować — a znalezisko,
//! które nie wywraca wywołania, przechodzi niezauważone.

use crate::core::config::MirrorConfig;
use crate::core::graph::{closure, Closure};
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};

pub fn execute(config_path: &Path, dir: &str, format: &str) -> Result<()> {
    let (root, config) = load(config_path)?;
    let dir = relative_to_root(&root, dir)?;
    let closure = closure(&root, &config, &dir)?;

    match format {
        "json" => println!("{}", serde_json::to_string_pretty(&closure)?),
        _ => print_closure(&closure),
    }
    Ok(())
}

fn print_closure(closure: &Closure) {
    println!(
        "{} ({})",
        closure.root_unit.bold(),
        closure.language.to_string().cyan()
    );
    println!("  {} krat/pakietów:", closure.units.len());
    for unit in &closure.units {
        println!("    {unit}");
    }
    println!("  {} repozytoriów:", closure.repos.len());
    for (path, git) in &closure.repos {
        println!("    {path}  {}", git.dimmed());
    }
}

/// Ścieżka podana przez użytkownika → ścieżka względem korzenia workspace'u.
/// Przyjmuje jedno i drugie, bo w praktyce woła się to z katalogu appki.
pub fn relative_to_root(root: &Path, dir: &str) -> Result<String> {
    let absolute = if Path::new(dir).is_absolute() {
        PathBuf::from(dir)
    } else {
        let from_cwd = std::env::current_dir()?.join(dir);
        if from_cwd.exists() {
            from_cwd
        } else {
            root.join(dir)
        }
    };
    let canonical = absolute
        .canonicalize()
        .with_context(|| format!("nie ma katalogu {dir}"))?;
    let rel = canonical.strip_prefix(root).map_err(|_| {
        anyhow!(
            "{} leży poza workspace'em {}",
            canonical.display(),
            root.display()
        )
    })?;
    Ok(rel.to_string_lossy().replace('\\', "/"))
}

pub fn load(config_path: &Path) -> Result<(PathBuf, MirrorConfig)> {
    let config = MirrorConfig::load(config_path)
        .with_context(|| format!("nie można wczytać {}", config_path.display()))?;
    let root = config
        .config_path
        .canonicalize()
        .with_context(|| format!("nie można rozwinąć {}", config_path.display()))?
        .parent()
        .ok_or_else(|| anyhow!("plik konfiguracyjny nie ma katalogu nadrzędnego"))?
        .to_path_buf();
    Ok((root, config))
}
