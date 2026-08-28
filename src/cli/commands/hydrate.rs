//! `mctl hydrate <katalog>` — montuje DOKŁADNIE domknięcie katalogu.
//!
//! Kurczak i jajko: domknięcie liczy się z prawdziwego workspace'u, a na
//! świeżej maszynie workspace'u jeszcze nie ma. Rozwiązanie: klonowanie do
//! punktu stałego — sklonuj repo katalogu, policz graf tym, co już jest,
//! dołóż nowo odkryte repozytoria, powtórz. Nierozwiązane ścieżki w trakcie
//! NIE są błędem; są nim dopiero, gdy runda nie przyniosła nowych repo.
//!
//! Praca nad jedną rodziną nie wymaga więc pełnego `sync` dwustu trzydziestu
//! repozytoriów.

use crate::cli::commands::{print_error, print_success};
use crate::core::config::MirrorConfig;
use crate::core::error::ConfigError;
use crate::core::graph::{closure, GraphError};
use crate::core::repository::Repository;
use crate::git::GitClient;
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Maksymalna liczba rund. Każda runda odkrywa co najmniej jedną warstwę
/// zależności; realne domknięcia mają 2–4 warstwy.
const MAX_ROUNDS: usize = 12;

pub fn execute(config_path: &Path, dir: &str) -> Result<()> {
    let git = GitClient::new();
    git.check_git_available()
        .map_err(|e| anyhow!("git niedostępny: {e}"))?;

    let config = load_with_bootstrap(config_path, &git)?;
    let root = config
        .config_path
        .canonicalize()?
        .parent()
        .ok_or_else(|| anyhow!("plik konfiguracyjny nie ma katalogu nadrzędnego"))?
        .to_path_buf();

    // Ścieżka podana WZGLĘDEM KORZENIA, bo katalogu może jeszcze nie być na
    // dysku — po to właśnie wołamy hydrate.
    let dir = dir.trim_start_matches("./").trim_end_matches('/').to_string();

    let mut cloned: BTreeSet<String> = BTreeSet::new();

    // Pliki korzenia (flat) idą pierwsze: bez pnpm-workspace.yaml i root
    // Cargo.toml nie ma z czego liczyć grafu.
    for repo in config.repositories.iter().filter(|r| r.flat) {
        ensure_cloned(&git, &root, repo, &mut cloned)?;
    }

    // Repo pokrywające wskazany katalog — od niego zaczyna się graf.
    let mut owners: Vec<&Repository> = config
        .repositories
        .iter()
        .filter(|r| dir == r.path || dir.starts_with(&format!("{}/", r.path)))
        .collect();
    owners.sort_by_key(|r| std::cmp::Reverse(r.path.len()));
    let owner = owners
        .first()
        .ok_or_else(|| anyhow!("żadne repozytorium nie pokrywa ścieżki '{dir}'"))?;
    ensure_cloned(&git, &root, owner, &mut cloned)?;

    for round in 1..=MAX_ROUNDS {
        match closure(&root, &config, &dir) {
            Ok(graph) => {
                let mut added = 0;
                for path in graph.repos.keys() {
                    let repo = config
                        .repositories
                        .iter()
                        .find(|r| &r.path == path)
                        .ok_or_else(|| anyhow!("repozytorium '{path}' zniknęło z manifestu"))?;
                    if ensure_cloned(&git, &root, repo, &mut cloned)? {
                        added += 1;
                    }
                }
                if added == 0 {
                    print_success(&format!(
                        "domknięcie {} zmontowane: {} repozytoriów, {} krat/pakietów",
                        dir.bold(),
                        graph.repos.len(),
                        graph.units.len()
                    ));
                    for path in graph.repos.keys() {
                        println!("  {path}");
                    }
                    return Ok(());
                }
            }
            Err(error) => {
                // Brakujące repozytorium wskazuje SAM BŁĄD — celujemy w nie
                // wprost, zamiast zgadywać po sąsiedztwie ścieżek.
                let added = clone_for_error(&git, &root, &config, &error, &mut cloned)?;
                if added == 0 {
                    print_error(&format!("nie da się domknąć grafu: {error}"));
                    return Err(anyhow!("{error}"));
                }
            }
        }
        if round == MAX_ROUNDS {
            return Err(anyhow!(
                "domknięcie nie ustabilizowało się po {MAX_ROUNDS} rundach"
            ));
        }
    }
    Ok(())
}

/// Wczytuje konfigurację, dociągając po drodze repozytoria z manifestami.
///
/// Na świeżym runnerze `mirror.toml` korzenia wskazuje na manifesty rodzin,
/// których repozytorium jeszcze nie ma — tak samo jak przy `mctl sync`,
/// który ma na to własny bootstrap. Klonujemy repozytoria zadeklarowane
/// WPROST w pliku korzenia i ponawiamy wczytanie.
fn load_with_bootstrap(config_path: &Path, git: &GitClient) -> Result<MirrorConfig> {
    match MirrorConfig::load(config_path) {
        Ok(config) => Ok(config),
        Err(ConfigError::IncludeNotFound { .. }) => {
            let raw = MirrorConfig::load_raw(config_path)
                .with_context(|| format!("nie można wczytać {}", config_path.display()))?;
            let root = config_path
                .canonicalize()?
                .parent()
                .ok_or_else(|| anyhow!("plik konfiguracyjny nie ma katalogu nadrzędnego"))?
                .to_path_buf();
            let mut cloned = BTreeSet::new();
            for repo in &raw.repositories {
                ensure_cloned(git, &root, repo, &mut cloned)?;
            }
            MirrorConfig::load(config_path)
                .with_context(|| format!("nie można wczytać {}", config_path.display()))
        }
        Err(e) => Err(anyhow!("nie można wczytać {}: {e}", config_path.display())),
    }
}

/// Klonuje repozytorium wskazane przez BŁĄD grafu.
///
/// `DeadPath` niesie rozwiniętą ścieżkę — wystarczy znaleźć repozytorium,
/// które ją pokrywa. `UnknownWorkspacePackage` nie zna ścieżki (pakiet nie
/// ma jeszcze package.json na dysku), więc dociągamy repozytoria pokrywające
/// wpisy pnpm-workspace.yaml, których brakuje. W obu razach zbiór jest
/// ograniczony — nigdy „sklonuj wszystko".
fn clone_for_error(
    git: &GitClient,
    root: &Path,
    config: &MirrorConfig,
    error: &GraphError,
    cloned: &mut BTreeSet<String>,
) -> Result<usize> {
    let mut targets: Vec<String> = Vec::new();

    match error {
        GraphError::DeadPath { resolved, .. } => {
            if let Some(rel) = relative(root, resolved) {
                targets.push(rel);
            }
        }
        GraphError::UnknownWorkspacePackage { .. } | GraphError::Io { .. } => {
            targets.extend(missing_workspace_entries(root));
        }
        _ => {}
    }

    let mut added = 0;
    for target in targets {
        // Najdłuższy pasujący prefiks: ścieżka kraty leży wewnątrz repo.
        let mut candidates: Vec<&Repository> = config
            .repositories
            .iter()
            .filter(|r| target == r.path || target.starts_with(&format!("{}/", r.path)))
            .collect();
        candidates.sort_by_key(|r| std::cmp::Reverse(r.path.len()));
        if let Some(repo) = candidates.first() {
            if ensure_cloned(git, root, repo, cloned)? {
                added += 1;
            }
        }
    }
    Ok(added)
}

/// Wpisy pnpm-workspace.yaml korzenia, których katalogów nie ma na dysku.
fn missing_workspace_entries(root: &Path) -> Vec<String> {
    let Ok(content) = std::fs::read_to_string(root.join("pnpm-workspace.yaml")) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        let Some(entry) = line.strip_prefix("- ") else { continue };
        let entry = entry.trim_matches('"').trim_matches('\'').trim_end_matches("/*");
        if !root.join(entry).exists() {
            out.push(entry.to_string());
        }
    }
    out
}

/// Ścieżka względem korzenia, znormalizowana LEKSYKALNIE.
///
/// `canonicalize` tu nie zadziała — cel jeszcze nie istnieje, bo właśnie po to
/// go szukamy. Segmenty `..` zwijamy sami, inaczej „xbooks-api/../../korea"
/// dopasowałoby się do repozytorium appki i pętla stanęłaby w miejscu.
fn relative(root: &Path, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(root).ok()?;
    let rel = rel.to_string_lossy().into_owned();
    let mut parts: Vec<&str> = Vec::new();
    for component in rel.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            other => parts.push(other),
        }
    }
    Some(parts.join("/"))
}

/// Klonuje repozytorium, jeśli go nie ma. Zwraca `true`, gdy coś doszło.
fn ensure_cloned(
    git: &GitClient,
    root: &Path,
    repo: &Repository,
    cloned: &mut BTreeSet<String>,
) -> Result<bool> {
    if !cloned.insert(repo.path.clone()) {
        return Ok(false);
    }
    let local: PathBuf = root.join(&repo.path);
    // Katalog `flat` (pliki korzenia) ISTNIEJE zawsze — to sam korzeń — więc
    // o pominięciu decyduje wyłącznie zbiór już obsłużonych repozytoriów
    // wyżej; kopiowanie jest idempotentne (nie nadpisuje).
    if !repo.flat && local.join(".git").exists() {
        return Ok(false);
    }
    if let Some(parent) = local.parent() {
        std::fs::create_dir_all(parent)?;
    }
    println!("  {} {}", "klonuję".dimmed(), repo.path);

    // `flat` to pliki korzenia BEZ historii: klon idzie do katalogu
    // tymczasowego, a stamtąd kopiujemy zawartość bez `.git` — dokładnie tak
    // jak robi to `mctl sync`.
    if repo.flat {
        let tmp = std::env::temp_dir().join(format!(
            "mctl-hydrate-flat-{}",
            repo.path.replace(['/', '.'], "_")
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        git.clone(&repo.git, &tmp, &repo.version_spec())
            .map_err(|e| anyhow!("klon {} nie powiódł się: {e}", repo.path))?;
        std::fs::create_dir_all(&local)?;
        copy_flat(&tmp, &local)?;
        let _ = std::fs::remove_dir_all(&tmp);
        return Ok(true);
    }

    git.clone(&repo.git, &local, &repo.version_spec())
        .map_err(|e| anyhow!("klon {} nie powiódł się: {e}", repo.path))?;
    Ok(true)
}

/// Kopiuje zawartość klonu bez `.git`, nie nadpisując istniejących plików.
fn copy_flat(from: &Path, to: &Path) -> Result<()> {
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let name = entry.file_name();
        if name == ".git" {
            continue;
        }
        let target = to.join(&name);
        if entry.path().is_dir() {
            std::fs::create_dir_all(&target)?;
            copy_flat(&entry.path(), &target)?;
        } else if !target.exists() {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}
