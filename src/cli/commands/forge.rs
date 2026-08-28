//! `mctl graph` / `mctl context` / `mctl build` — obrazy z workspace'u.
//!
//! W odróżnieniu od reszty komend te trzy KOŃCZĄ SIĘ NIEZEROWO przy błędzie.
//! Martwa ścieżka w grafie albo nieudany build muszą wywrócić CI, a nie
//! przejść jako sukces — dzisiejsze ciche gnicie kontekstów obrazów wzięło
//! się dokładnie z połykania błędów.

use crate::core::config::MirrorConfig;
use crate::core::forge::{assemble_context, build_image, ForgeReceipt};
use crate::core::graph::{image_graph, ImageGraph};
use crate::core::image::ImageSpec;
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};

/// Wypisuje domknięcie zależności obrazu.
pub fn graph(config_path: &Path, image: &str, format: &str) -> Result<()> {
    let (root, config, spec) = load(config_path, image)?;
    let graph = image_graph(&root, &config, &spec)?;

    match format {
        "json" => println!("{}", serde_json::to_string_pretty(&graph)?),
        _ => print_graph(&graph),
    }
    Ok(())
}

/// Składa kontekst budowania w katalogu wyjściowym.
pub fn context(config_path: &Path, image: &str, out: &Path) -> Result<()> {
    let (root, config, spec) = load(config_path, image)?;
    let graph = image_graph(&root, &config, &spec)?;
    let receipt = assemble_context(&root, &spec, &graph, out)?;

    println!(
        "{} kontekst {} w {}",
        "✓".green(),
        spec.name.bold(),
        out.display()
    );
    println!(
        "  {} krat/pakietów z {} repozytoriów",
        graph.units.len(),
        receipt.repos.len()
    );
    Ok(())
}

/// Składa kontekst i buduje obraz.
#[allow(clippy::too_many_arguments)]
pub fn build(
    config_path: &Path,
    image: &str,
    tag: &str,
    push: bool,
    load_image: bool,
    no_cache_store: bool,
    keep_context: Option<PathBuf>,
) -> Result<()> {
    let (root, config, spec) = load(config_path, image)?;
    let graph = image_graph(&root, &config, &spec)?;

    let out = match &keep_context {
        Some(path) => path.clone(),
        None => std::env::temp_dir().join(format!("mctl-forge-{}", spec.name)),
    };
    let receipt = assemble_context(&root, &spec, &graph, &out)?;
    println!(
        "{} kontekst złożony ({} krat/pakietów, {} repozytoriów)",
        "→".cyan(),
        graph.units.len(),
        receipt.repos.len()
    );

    build_image(&out, &spec, &receipt, tag, push, load_image, no_cache_store)?;

    if keep_context.is_none() {
        let _ = std::fs::remove_dir_all(&out);
    }
    println!("{} {}:{}", "✓".green(), spec.registry.bold(), tag);
    Ok(())
}

/// Lista zadeklarowanych obrazów.
pub fn list(config_path: &Path) -> Result<()> {
    let (_, config, _) = match load_config(config_path) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };
    if config.images.is_empty() {
        println!("Żaden manifest nie deklaruje [[images]].");
        return Ok(());
    }
    for image in &config.images {
        println!(
            "{:<28} {:<10} {}",
            image.name.bold(),
            image.kind.to_string().cyan(),
            image.app
        );
    }
    Ok(())
}

fn print_graph(graph: &ImageGraph) {
    println!("{} ({})", graph.image.bold(), graph.kind.to_string().cyan());
    println!("  app: {}", graph.app);
    println!("  {} krat/pakietów:", graph.units.len());
    for unit in &graph.units {
        println!("    {unit}");
    }
    println!("  {} repozytoriów:", graph.repos.len());
    for (path, git) in &graph.repos {
        println!("    {path}  {}", git.dimmed());
    }
}

type Loaded = (PathBuf, MirrorConfig, ImageSpec);

fn load(config_path: &Path, image: &str) -> Result<Loaded> {
    let (root, config, _) = load_config(config_path)?;
    let spec = config
        .images
        .iter()
        .find(|i| i.name == image)
        .cloned()
        .ok_or_else(|| {
            let known: Vec<&str> = config.images.iter().map(|i| i.name.as_str()).collect();
            anyhow!(
                "obraz '{}' nie jest zadeklarowany w żadnym manifeście; znane: {}",
                image,
                if known.is_empty() {
                    "(brak)".to_string()
                } else {
                    known.join(", ")
                }
            )
        })?;
    spec.validate().map_err(|e| anyhow!(e))?;
    Ok((root, config, spec))
}

fn load_config(config_path: &Path) -> Result<Loaded> {
    let config = MirrorConfig::load(config_path)
        .with_context(|| format!("nie można wczytać {}", config_path.display()))?;
    let root = config
        .config_path
        .canonicalize()
        .with_context(|| format!("nie można rozwinąć {}", config_path.display()))?
        .parent()
        .ok_or_else(|| anyhow!("plik konfiguracyjny nie ma katalogu nadrzędnego"))?
        .to_path_buf();
    // Pierwszy obraz służy tylko za wypełniacz w krotce; wywołania listujące
    // go ignorują.
    let placeholder = config
        .images
        .first()
        .cloned()
        .unwrap_or(ImageSpec {
            name: String::new(),
            app: String::new(),
            kind: crate::core::image::ImageKind::RustBin,
            port: 0,
            registry: String::new(),
            bin: None,
            cmd: None,
            env: Vec::new(),
        });
    Ok((root, config, placeholder))
}

/// Kwit z ostatniego kontekstu — do wypisania w CI.
pub fn print_receipt(receipt: &ForgeReceipt) {
    for (repo, head) in &receipt.repos {
        println!("{repo} {head}");
    }
}
