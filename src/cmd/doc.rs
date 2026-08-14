//! `cart doc` — generate Markdown documentation from doc comments.

use crate::manifest::CartManifest;
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

pub fn doc(manifest_path: &Path) -> Result<()> {
    let manifest = CartManifest::load(manifest_path)?;
    let project_name = &manifest.package.name;

    let output_dir = manifest
        .doc
        .as_ref()
        .and_then(|d| d.output.as_deref())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            manifest_path
                .parent()
                .unwrap_or(Path::new("."))
                .join("target")
                .join("doc")
                .join(project_name)
        });

    fs::create_dir_all(&output_dir)?;

    let root_file = if let Some(bank) = &manifest.bank {
        manifest_path
            .parent()
            .unwrap_or(Path::new("."))
            .join(bank.path.as_deref().unwrap_or("src/bank.op"))
    } else if let Some(rom) = manifest.rom.first() {
        manifest_path
            .parent()
            .unwrap_or(Path::new("."))
            .join(rom.path.as_deref().unwrap_or("src/cart.op"))
    } else {
        return Err(anyhow::anyhow!("E502: no bank or rom target in Cart.toml"));
    };

    let mut modules: Vec<(String, PathBuf)> = Vec::new();
    walk_module(&root_file, &output_dir, &mut modules)?;

    let mut index = String::new();
    index.push_str(&format!("# {project_name}\n\n"));
    if let Some(bank) = &manifest.bank {
        index.push_str(&format!("Bank: {}\n\n", bank.name));
    }
    for rom in &manifest.rom {
        index.push_str(&format!("ROM: {} (target: {})\n\n", rom.name, rom.target));
    }
    index.push_str("## Modules\n\n");
    for (name, _) in &modules {
        index.push_str(&format!("- [{name}]({name}.md)\n"));
    }
    fs::write(output_dir.join("index.md"), index)?;

    eprintln!("Documentation written to {}", output_dir.display());
    Ok(())
}

fn walk_module(file: &Path, output_dir: &Path, modules: &mut Vec<(String, PathBuf)>) -> Result<()> {
    if !file.exists() {
        return Ok(());
    }

    let source = fs::read_to_string(file)?;
    let module_name = file
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "module".to_string());

    let mut md = String::new();
    md.push_str(&format!("# Module: {module_name}\n\n"));

    let mut in_doc = false;
    let mut doc_text = String::new();
    let mut module_docs = String::new();
    let mut in_module_doc = false;

    for line in source.lines() {
        let trimmed = line.trim_start();
        if let Some(stripped) = trimmed.strip_prefix("//!") {
            let text = stripped.trim_start();
            module_docs.push_str(text);
            module_docs.push('\n');
            in_module_doc = true;
            continue;
        }
        if in_module_doc && !trimmed.starts_with("//!") {
            in_module_doc = false;
        }

        if let Some(stripped) = trimmed.strip_prefix("///") {
            let text = stripped.trim_start();
            doc_text.push_str(text);
            doc_text.push('\n');
            in_doc = true;
            continue;
        }

        if in_doc && !trimmed.starts_with("///") {
            in_doc = false;
            let decl = extract_decl_name(trimmed);
            if let Some(name) = decl {
                md.push_str(&format!("## {name}\n\n"));
                md.push_str(&doc_text);
                md.push('\n');
                doc_text.clear();
            }
        }

        if trimmed.starts_with("mod ") && trimmed.ends_with(';') {
            let mod_name = trimmed
                .trim_start_matches("mod ")
                .trim_end_matches(';')
                .trim();
            let mod_file = file
                .parent()
                .map(|d| d.join(format!("{mod_name}.op")))
                .unwrap_or_else(|| PathBuf::from(format!("{mod_name}.op")));
            if mod_file.exists() {
                walk_module(&mod_file, output_dir, modules)?;
            }
        }
    }

    if !module_docs.is_empty() {
        let preamble = format!("{module_docs}\n");
        md = format!("{preamble}{md}");
    }

    let out_path = output_dir.join(format!("{module_name}.md"));
    fs::write(&out_path, md)?;
    modules.push((module_name, out_path));

    Ok(())
}

fn extract_decl_name(line: &str) -> Option<String> {
    let keywords = [
        "const ",
        "fn ",
        "inline fn ",
        "struct ",
        "enum ",
        "type ",
        "mod ",
    ];
    for kw in &keywords {
        if let Some(rest) = line.strip_prefix(kw) {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}
