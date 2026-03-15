#!/usr/bin/env rust-script
//! Rebranding tool: redclaw → redclaw
//!
//! Usage:
//!   rust-script dev/rebrand.rs --dry-run    # Preview changes
//!   rust-script dev/rebrand.rs --execute    # Apply changes
//!
//! This tool replaces all instances of redclaw/RedClaw/REDCLAW
//! with redclaw/RedClaw/REDCLAW across the entire repository.

use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

// ── Configuration ──────────────────────────────────────────────

const FROM_PATTERNS: &[(&str, &str)] = &[
    // Exact case matches
    ("redclaw", "redclaw"),
    ("RedClaw", "RedClaw"),
    ("REDCLAW", "REDCLAW"),
    ("redclaw-labs", "redclaw-labs"),
    ("RedClaw-Labs", "RedClaw-Labs"),
    ("RedClawLabs", "RedClawLabs"),
    ("redclawlabs", "redclawlabs"),
    // Paths and directories
    ("~/.redclaw", "~/.redclaw"),
    ("/.redclaw/", "/.redclaw/"),
    (".redclaw/", ".redclaw/"),
    ("redclaw/", "redclaw/"),
    // URLs
    ("github.com/redclaw-labs", "github.com/redclaw-labs"),
    ("redclaw-labs.github.io", "redclaw-labs.github.io"),
    ("redclawlabs.ai", "redclawlabs.ai"),
    // Package names (Python)
    ("redclaw-tools", "redclaw-tools"),
    ("redclaw_tools", "redclaw_tools"),
    // Environment variables
    ("REDCLAW_", "REDCLAW_"),
    // Config keys
    ("[redclaw]", "[redclaw]"),
];

// Files/directories to skip
const SKIP_DIRS: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    "__pycache__",
    ".venv",
    "venv",
    "dist",
    "build",
    "backups",
];

const SKIP_FILES: &[&str] = &[
    "Cargo.lock",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "rebrand.rs",
    "rebrand.log",
];

const SKIP_EXTENSIONS: &[&str] = &[
    ".png", ".jpg", ".jpeg", ".gif", ".ico", ".svg", ".pdf", ".bin", ".exe", ".dll", ".so",
    ".dylib",
];

// ── Data Structures ──────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Replacement {
    line_number: usize,
    column: usize,
    from: String,
    to: String,
    context: String,
}

#[derive(Debug, Default)]
struct FileReport {
    path: PathBuf,
    replacements: Vec<Replacement>,
    backup_path: Option<PathBuf>,
    error: Option<String>,
}

#[derive(Debug, Default)]
struct Summary {
    total_files_scanned: usize,
    total_files_modified: usize,
    total_replacements: usize,
    replacements_by_pattern: HashMap<String, usize>,
    errors: Vec<String>,
    skipped_files: HashSet<String>,
}

// ── Main Logic ──────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let dry_run = args.iter().any(|a| a == "--dry-run" || a == "-n");
    let execute = args.iter().any(|a| a == "--execute" || a == "-e");
    let help = args.iter().any(|a| a == "--help" || a == "-h");

    if help {
        print_help();
        return;
    }

    if !dry_run && !execute {
        eprintln!("❌ Error: Must specify --dry-run or --execute");
        eprintln!();
        print_help();
        std::process::exit(1);
    }

    println!("🔄 RedClaw → RedClaw Rebranding Tool");
    println!("═══════════════════════════════════════");
    println!();

    if dry_run {
        println!("📋 Mode: DRY RUN (no changes will be made)");
    } else {
        println!("⚠️  Mode: EXECUTE (changes WILL be applied)");
        println!("⚠️  Backups will be created in dev/backups/");
    }
    println!();

    let start = Instant::now();

    // Determine repo root (parent of dev/)
    let repo_root = std::env::current_dir()
        .expect("Failed to get current directory")
        .parent()
        .expect("Failed to get parent directory")
        .to_path_buf();

    println!("📁 Repository root: {}", repo_root.display());
    println!();

    // Create backup directory if executing
    let backup_dir = repo_root.join("dev").join("backups");
    if execute && !backup_dir.exists() {
        fs::create_dir_all(&backup_dir).expect("Failed to create backup directory");
        println!("📦 Created backup directory: {}", backup_dir.display());
    }

    // Find all files
    println!("🔍 Scanning repository...");
    let mut files_to_process = Vec::new();
    collect_files(&repo_root, &mut files_to_process, 0);

    println!("   Found {} files to process", files_to_process.len());
    println!();

    // Process files
    println!("📝 Processing files...");
    let mut summary = Summary::default();
    let mut reports = Vec::new();

    for file_path in files_to_process {
        let report = process_file(&file_path, &repo_root, &backup_dir, dry_run, execute);

        if report.error.is_some() {
            summary.errors.push(format!(
                "{}: {}",
                file_path.display(),
                report.error.as_ref().unwrap()
            ));
        } else if !report.replacements.is_empty() {
            summary.total_files_modified += 1;
            summary.total_replacements += report.replacements.len();

            for replacement in &report.replacements {
                *summary
                    .replacements_by_pattern
                    .entry(replacement.from.clone())
                    .or_insert(0) += 1;
            }
        }

        if let Some(ref backup) = report.backup_path {
            println!(
                "   💾 Backed up: {} → {}",
                file_path.display(),
                backup.display()
            );
        }

        reports.push(report);
        summary.total_files_scanned += 1;
    }

    // Generate report
    println!();
    println!("═══════════════════════════════════════");
    println!("📊 SUMMARY");
    println!("═══════════════════════════════════════");
    println!("⏱️  Duration: {:.2?}", start.elapsed());
    println!("📁 Files scanned: {}", summary.total_files_scanned);
    println!("✏️  Files modified: {}", summary.total_files_modified);
    println!("🔄 Total replacements: {}", summary.total_replacements);
    println!();

    if !summary.replacements_by_pattern.is_empty() {
        println!("📈 Replacements by pattern:");
        let mut patterns: Vec<_> = summary.replacements_by_pattern.iter().collect();
        patterns.sort_by(|a, b| b.1.cmp(a.1));
        for (pattern, count) in patterns.iter().take(20) {
            println!("   {:<30} → {:>6}", pattern, count);
        }
        if patterns.len() > 20 {
            println!("   ... and {} more patterns", patterns.len() - 20);
        }
        println!();
    }

    // Write detailed report
    let report_path = repo_root.join("dev").join("rebrand_report.md");
    write_report(&reports, &summary, &report_path, dry_run);
    println!("📄 Detailed report: {}", report_path.display());
    println!();

    if !summary.errors.is_empty() {
        println!("❌ Errors ({}):", summary.errors.len());
        for error in summary.errors.iter().take(10) {
            println!("   {}", error);
        }
        if summary.errors.len() > 10 {
            println!("   ... and {} more errors", summary.errors.len() - 10);
        }
        println!();
    }

    if dry_run {
        println!(
            "✅ Dry run complete. Review the report, then run with --execute to apply changes."
        );
    } else {
        println!("✅ Rebranding complete! Review the report for details.");
        println!();
        println!("⚠️  IMPORTANT NEXT STEPS:");
        println!("   1. Review dev/rebrand_report.md for all changes");
        println!("   2. Run: cargo build --release");
        println!("   3. Run: cargo test");
        println!("   4. Update any external references (docs, configs, etc.)");
        println!("   5. Backups are stored in: dev/backups/");
    }
}

fn print_help() {
    println!("RedClaw → RedClaw Rebranding Tool");
    println!();
    println!("USAGE:");
    println!("    rust-script dev/rebrand.rs [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --dry-run, -n    Preview changes without modifying files");
    println!("    --execute, -e    Apply changes (creates backups)");
    println!("    --help, -h       Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    # Preview all changes");
    println!("    rust-script dev/rebrand.rs --dry-run");
    println!();
    println!("    # Apply all changes with backups");
    println!("    rust-script dev/rebrand.rs --execute");
    println!();
}

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>, depth: usize) {
    if depth > 10 {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Skip directories
        if path.is_dir() {
            if SKIP_DIRS.contains(&file_name) {
                continue;
            }
            collect_files(&path, files, depth + 1);
            continue;
        }

        // Skip files
        if SKIP_FILES.contains(&file_name) {
            continue;
        }

        // Skip extensions
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if SKIP_EXTENSIONS.contains(&ext) {
                continue;
            }
        }

        files.push(path);
    }
}

fn process_file(
    file_path: &Path,
    repo_root: &Path,
    backup_dir: &Path,
    dry_run: bool,
    execute: bool,
) -> FileReport {
    let mut report = FileReport {
        path: file_path.to_path_buf(),
        ..Default::default()
    };

    // Read file
    let Ok(content) = fs::read_to_string(file_path) else {
        report.error = Some("Failed to read file".to_string());
        return report;
    };

    // Check if file contains any patterns
    let mut has_matches = false;
    for (from, _) in FROM_PATTERNS {
        if content.contains(from) {
            has_matches = true;
            break;
        }
    }

    if !has_matches {
        return report;
    }

    // Process line by line
    let mut new_content = String::new();
    let mut lines_modified = false;

    for (line_idx, line) in content.lines().enumerate() {
        let mut new_line = line.to_string();
        let mut line_modified = false;

        for (from, to) in FROM_PATTERNS {
            if new_line.contains(from) {
                // Find all occurrences
                let mut start_idx = 0;
                while let Some(pos) = new_line[start_idx..].find(from) {
                    let actual_pos = start_idx + pos;

                    // Record replacement
                    report.replacements.push(Replacement {
                        line_number: line_idx + 1,
                        column: actual_pos + 1,
                        from: from.to_string(),
                        to: to.to_string(),
                        context: get_context(&new_line, actual_pos, 40),
                    });

                    // Apply replacement
                    new_line.replace_range(actual_pos..actual_pos + from.len(), to);
                    start_idx = actual_pos + to.len();
                    line_modified = true;
                    lines_modified = true;
                }
            }
        }

        new_content.push_str(&new_line);
        new_content.push('\n');
    }

    // Remove trailing newline if original didn't have one
    if !content.ends_with('\n') && new_content.ends_with('\n') {
        new_content.pop();
    }

    if lines_modified {
        if dry_run {
            // Just report, don't write
        } else if execute {
            // Create backup
            let relative_path = file_path
                .strip_prefix(repo_root)
                .unwrap_or(file_path.as_path());
            let backup_path = backup_dir.join(relative_path);

            if let Some(parent) = backup_path.parent() {
                fs::create_dir_all(parent).ok();
            }

            if fs::copy(file_path, &backup_path).is_ok() {
                report.backup_path = Some(backup_path);
            }

            // Write new content
            if let Ok(mut file) = File::create(file_path) {
                if file.write_all(new_content.as_bytes()).is_ok() {
                    // Success
                } else {
                    report.error = Some("Failed to write file".to_string());
                }
            } else {
                report.error = Some("Failed to create file".to_string());
            }
        }
    }

    report
}

fn get_context(line: &str, pos: usize, context_size: usize) -> String {
    let start = pos.saturating_sub(context_size / 2);
    let end = (pos + context_size / 2).min(line.len());

    let context = &line[start..end];

    if start > 0 && end < line.len() {
        format!("...{}...", context)
    } else if start > 0 {
        format!("...{}", context)
    } else if end < line.len() {
        format!("{}...", context)
    } else {
        context.to_string()
    }
}

fn write_report(reports: &[FileReport], summary: &Summary, report_path: &Path, dry_run: bool) {
    let mut output = String::new();

    output.push_str("# Rebranding Report: RedClaw → RedClaw\n\n");
    output.push_str(&format!(
        "**Mode:** {}\n\n",
        if dry_run { "DRY RUN" } else { "EXECUTE" }
    ));

    output.push_str("## Summary\n\n");
    output.push_str(&format!(
        "- Files scanned: {}\n",
        summary.total_files_scanned
    ));
    output.push_str(&format!(
        "- Files modified: {}\n",
        summary.total_files_modified
    ));
    output.push_str(&format!(
        "- Total replacements: {}\n\n",
        summary.total_replacements
    ));

    if !summary.replacements_by_pattern.is_empty() {
        output.push_str("## Replacements by Pattern\n\n");
        output.push_str("| Pattern | Count |\n");
        output.push_str("|---------|-------|\n");

        let mut patterns: Vec<_> = summary.replacements_by_pattern.iter().collect();
        patterns.sort_by(|a, b| b.1.cmp(a.1));

        for (pattern, count) in patterns {
            output.push_str(&format!("| `{}` | {} |\n", pattern, count));
        }
        output.push('\n');
    }

    output.push_str("## Modified Files\n\n");

    let modified_reports: Vec<_> = reports
        .iter()
        .filter(|r| !r.replacements.is_empty())
        .collect();

    for report in modified_reports.iter().take(100) {
        output.push_str(&format!("### {}\n\n", report.path.display()));

        if let Some(ref backup) = report.backup_path {
            output.push_str(&format!("**Backup:** `{}`\n\n", backup.display()));
        }

        output.push_str("| Line | Column | From | To | Context |\n");
        output.push_str("|------|--------|------|-----|---------|\n");

        for replacement in &report.replacements {
            output.push_str(&format!(
                "| {} | {} | `{}` | `{}` | `{}` |\n",
                replacement.line_number,
                replacement.column,
                replacement.from,
                replacement.to,
                replacement.context.replace('|', "\\|")
            ));
        }
        output.push('\n');
    }

    if modified_reports.len() > 100 {
        output.push_str(&format!(
            "\n... and {} more files (see full report in tool output)\n",
            modified_reports.len() - 100
        ));
    }

    if !summary.errors.is_empty() {
        output.push_str("\n## Errors\n\n");
        for error in &summary.errors {
            output.push_str(&format!("- {}\n", error));
        }
    }

    fs::write(report_path, output).ok();
}
