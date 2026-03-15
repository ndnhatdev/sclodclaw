//! Rebranding tool: zeroclaw → redclaw
//!
//! Usage:
//!   cargo run --release -- --dry-run    # Preview changes
//!   cargo run --release -- --execute    # Apply changes

use anyhow::{Context, Result};
use chrono::Local;
use clap::Parser;
use regex::Regex;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

// ── CLI Arguments ──────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(name = "rebrand-tool")]
#[command(about = "ZeroClaw → RedClaw rebranding tool")]
struct Args {
    /// Preview changes without modifying files
    #[arg(long, short = 'n')]
    dry_run: bool,

    /// Apply changes (creates backups)
    #[arg(long, short = 'e')]
    execute: bool,

    /// Path to repository root (default: parent of this tool)
    #[arg(long, default_value = "")]
    repo_root: String,
}

// ── Replacement Patterns ──────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Pattern {
    from: String,
    to: String,
    regex: Regex,
}

impl Pattern {
    fn new(from: &str, to: &str) -> Result<Self> {
        // Escape special regex characters except for word boundaries we might add
        let escaped = regex::escape(from);
        let regex = Regex::new(&escaped)
            .with_context(|| format!("Failed to compile regex for '{}'", from))?;

        Ok(Self {
            from: from.to_string(),
            to: to.to_string(),
            regex,
        })
    }
}

fn create_patterns() -> Vec<Pattern> {
    let patterns = vec![
        // Exact case matches - core branding
        ("zeroclaw", "redclaw"),
        ("ZeroClaw", "RedClaw"),
        ("ZEROCLAW", "REDCLAW"),
        // Organization/URL patterns
        ("zeroclaw-labs", "redclaw-labs"),
        ("ZeroClaw-Labs", "RedClaw-Labs"),
        ("ZeroClawLabs", "RedClawLabs"),
        ("zeroclawlabs", "redclawlabs"),
        // Paths and directories
        ("~/.zeroclaw", "~/.redclaw"),
        (".zeroclaw/", ".redclaw/"),
        // URLs
        ("github.com/zeroclaw-labs", "github.com/redclaw-labs"),
        ("zeroclaw-labs.github.io", "redclaw-labs.github.io"),
        ("zeroclawlabs.ai", "redclawlabs.ai"),
        // Package names (Python)
        ("zeroclaw-tools", "redclaw-tools"),
        ("zeroclaw_tools", "redclaw_tools"),
        // Environment variables
        ("ZEROCLAW_", "REDCLAW_"),
        // Config section headers
        ("[zeroclaw]", "[redclaw]"),
    ];

    patterns
        .into_iter()
        .filter_map(|(from, to)| Pattern::new(from, to).ok())
        .collect()
}

// ── Files/Directories to Skip ──────────────────────────────────────────────

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
    "rebrand-tool",
];

const SKIP_FILES: &[&str] = &[
    "Cargo.lock",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "rebrand.log",
];

const SKIP_EXTENSIONS: &[&str] = &[
    ".png", ".jpg", ".jpeg", ".gif", ".ico", ".svg", ".pdf", ".bin", ".exe", ".dll", ".so",
    ".dylib", ".woff", ".woff2", ".ttf", ".eot",
];

// ── Data Structures ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
struct Replacement {
    line_number: usize,
    column: usize,
    from: String,
    to: String,
    context: String,
}

#[derive(Debug, Default, Serialize)]
struct FileReport {
    path: String,
    replacements: Vec<Replacement>,
    backup_path: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Default, Serialize)]
struct Summary {
    total_files_scanned: usize,
    total_files_modified: usize,
    total_replacements: usize,
    replacements_by_pattern: HashMap<String, usize>,
    errors: Vec<String>,
    skipped_files: HashSet<String>,
    started_at: String,
    completed_at: String,
}

// ── Main Logic ──────────────────────────────────────────────

fn main() -> Result<()> {
    let args = Args::parse();

    if !args.dry_run && !args.execute {
        eprintln!("❌ Error: Must specify --dry-run or --execute");
        eprintln!();
        eprintln!("USAGE:");
        eprintln!("    cargo run --release -- --dry-run    # Preview changes");
        eprintln!("    cargo run --release -- --execute    # Apply changes");
        std::process::exit(1);
    }

    println!("🔄 ZeroClaw → RedClaw Rebranding Tool");
    println!("═══════════════════════════════════════");
    println!();

    if args.dry_run {
        println!("📋 Mode: DRY RUN (no changes will be made)");
    } else {
        println!("⚠️  Mode: EXECUTE (changes WILL be applied)");
        println!("⚠️  Backups will be created in dev/backups/");
    }
    println!();

    let start = Instant::now();

    // Determine repo root
    let repo_root = if args.repo_root.is_empty() {
        std::env::current_dir()?
            .parent()
            .context("Failed to get parent directory")?
            .to_path_buf()
    } else {
        PathBuf::from(&args.repo_root)
    };

    println!("📁 Repository root: {}", repo_root.display());
    println!();

    // Create backup directory if executing
    let backup_dir = repo_root.join("dev").join("backups");
    if args.execute && !backup_dir.exists() {
        fs::create_dir_all(&backup_dir).context("Failed to create backup directory")?;
        println!("📦 Created backup directory: {}", backup_dir.display());
    }

    // Create patterns
    let patterns = create_patterns();
    println!("📝 Loaded {} replacement patterns", patterns.len());
    println!();

    // Find all files
    println!("🔍 Scanning repository...");
    let mut files_to_process = Vec::new();
    collect_files(&repo_root, &mut files_to_process, 0);

    println!("   Found {} files to process", files_to_process.len());
    println!();

    // Process files
    println!("📝 Processing files...");
    let mut summary = Summary::default();
    summary.started_at = Local::now().to_rfc3339();
    let mut reports = Vec::new();

    for (idx, file_path) in files_to_process.iter().enumerate() {
        if idx % 100 == 0 {
            println!(
                "   Processing file {}/{}...",
                idx + 1,
                files_to_process.len()
            );
        }

        let report = process_file(
            file_path,
            &repo_root,
            &backup_dir,
            &patterns,
            args.dry_run,
            args.execute,
        )?;

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
            println!("   💾 Backed up: {} → {}", file_path.display(), backup);
        }

        reports.push(report);
        summary.total_files_scanned += 1;
    }

    summary.completed_at = Local::now().to_rfc3339();

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
    write_report(&reports, &summary, &report_path, args.dry_run)?;
    println!("📄 Detailed report: {}", report_path.display());
    println!();

    // Write JSON report
    let json_report_path = repo_root.join("dev").join("rebrand_report.json");
    fs::write(&json_report_path, serde_json::to_string_pretty(&summary)?)?;
    println!("📄 JSON report: {}", json_report_path.display());
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

    if args.dry_run {
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

    Ok(())
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
    patterns: &[Pattern],
    dry_run: bool,
    execute: bool,
) -> Result<FileReport> {
    let mut report = FileReport {
        path: file_path.display().to_string(),
        ..Default::default()
    };

    // Read file - skip binary files gracefully
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return Ok(report), // Skip binary/non-UTF8 files
    };

    // Check if file contains any patterns
    let mut has_matches = false;
    for pattern in patterns {
        if pattern.regex.is_match(&content) {
            has_matches = true;
            break;
        }
    }

    if !has_matches {
        return Ok(report);
    }

    // Process line by line
    let mut new_content = String::new();
    let mut lines_modified = false;

    for (line_idx, line) in content.lines().enumerate() {
        let mut new_line = line.to_string();
        let mut line_modified = false;

        for pattern in patterns {
            if pattern.regex.is_match(&new_line) {
                // Find all occurrences
                for mat in pattern.regex.find_iter(&new_line) {
                    // Record replacement
                    report.replacements.push(Replacement {
                        line_number: line_idx + 1,
                        column: mat.start() + 1,
                        from: pattern.from.clone(),
                        to: pattern.to.clone(),
                        context: get_context(&new_line, mat.start(), 40),
                    });
                }

                // Apply replacement
                new_line = pattern
                    .regex
                    .replace_all(&new_line, &pattern.to)
                    .to_string();
                line_modified = true;
                lines_modified = true;
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
            let relative_path = file_path.strip_prefix(repo_root).unwrap_or(file_path);
            let backup_path = backup_dir.join(relative_path);

            if let Some(parent) = backup_path.parent() {
                fs::create_dir_all(parent).ok();
            }

            if fs::copy(file_path, &backup_path).is_ok() {
                report.backup_path = Some(backup_path.display().to_string());
            }

            // Write new content
            let mut file = File::create(file_path).with_context(|| "Failed to create file")?;
            file.write_all(new_content.as_bytes())
                .with_context(|| "Failed to write file")?;
        }
    }

    Ok(report)
}

fn get_context(line: &str, byte_pos: usize, context_size: usize) -> String {
    // Convert byte position to char position safely
    let chars: Vec<char> = line.chars().collect();
    let char_pos = line[..byte_pos].chars().count();

    let start_char = char_pos.saturating_sub(context_size / 2);
    let end_char = (char_pos + context_size / 2).min(chars.len());

    let context: String = chars[start_char..end_char].iter().collect();

    if start_char > 0 && end_char < chars.len() {
        format!("...{}...", context)
    } else if start_char > 0 {
        format!("...{}", context)
    } else if end_char < chars.len() {
        format!("{}...", context)
    } else {
        context
    }
}

fn write_report(
    reports: &[FileReport],
    summary: &Summary,
    report_path: &Path,
    dry_run: bool,
) -> Result<()> {
    let mut output = String::new();

    output.push_str("# Rebranding Report: ZeroClaw → RedClaw\n\n");
    output.push_str(&format!(
        "**Mode:** {}\n\n",
        if dry_run { "DRY RUN" } else { "EXECUTE" }
    ));
    output.push_str(&format!("**Started:** {}\n", summary.started_at));
    output.push_str(&format!("**Completed:** {}\n\n", summary.completed_at));

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
        output.push_str(&format!("### {}\n\n", report.path));

        if let Some(ref backup) = report.backup_path {
            output.push_str(&format!("**Backup:** `{}`\n\n", backup));
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
            "\n... and {} more files (see JSON report for full list)\n",
            modified_reports.len() - 100
        ));
    }

    if !summary.errors.is_empty() {
        output.push_str("\n## Errors\n\n");
        for error in &summary.errors {
            output.push_str(&format!("- {}\n", error));
        }
    }

    fs::write(report_path, output).with_context(|| "Failed to write report")?;

    Ok(())
}
