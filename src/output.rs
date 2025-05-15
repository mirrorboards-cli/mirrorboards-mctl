use std::path::Path;

pub fn print_cloning(url: &str, path: &Path) {
    println!("Cloning {} into {}", url, path.display());
}

pub fn print_skipping(path: &Path) {
    println!("Skipping {}: directory already exists", path.display());
}

pub fn print_info(message: &str) {
    println!("{}", message);
}

pub fn print_summary(total: usize, cloned: usize, skipped: usize) {
    println!("\nSummary:");
    println!("Total repositories: {}", total);
    println!("Cloned: {}", cloned);
    println!("Skipped: {}", skipped);
}

/// Add color and style to terminal output
/// Supports: bold, red, green, yellow, blue, magenta, cyan, white
/// And combined styles like "bold blue"
pub fn colorize(text: &str, style: &str) -> String {
    // Support combined styles like "bold blue"
    if style.contains(' ') {
        let styles: Vec<&str> = style.split(' ').collect();
        let mut result = text.to_string();
        
        for s in styles {
            result = colorize(&result, s);
        }
        
        return result;
    }
    
    match style {
        "bold" => format!("\x1b[1m{}\x1b[0m", text),
        "red" => format!("\x1b[31m{}\x1b[0m", text),
        "green" => format!("\x1b[32m{}\x1b[0m", text),
        "yellow" => format!("\x1b[33m{}\x1b[0m", text),
        "blue" => format!("\x1b[34m{}\x1b[0m", text),
        "magenta" => format!("\x1b[35m{}\x1b[0m", text),
        "cyan" => format!("\x1b[36m{}\x1b[0m", text),
        "white" => format!("\x1b[37m{}\x1b[0m", text),
        _ => text.to_string(),
    }
}