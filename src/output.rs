use std::path::Path;

pub fn print_cloning(url: &str, path: &Path) {
    println!("Cloning {} into {}", url, path.display());
}

pub fn print_skipping(path: &Path) {
    println!("Skipping {}: directory already exists", path.display());
}

pub fn print_summary(total: usize, cloned: usize, skipped: usize) {
    println!("\nSummary:");
    println!("Total repositories: {}", total);
    println!("Cloned: {}", cloned);
    println!("Skipped: {}", skipped);
}