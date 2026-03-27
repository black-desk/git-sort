use clap::Parser;
use git_sort::{parse_commits, sort_by_topo_order};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::Command;

/// Sort commits by topological order
#[derive(Parser)]
#[command(version)]
struct Args {
    /// Input file containing commit hashes (one per line).
    /// Use '-' for stdin.
    #[arg(default_value = "-")]
    input: String,

    /// Output file. Use '-' for stdout.
    #[arg(short, long, default_value = "-")]
    output: String,

    /// Reference branch for topological ordering
    #[arg(long, default_value = "HEAD")]
    reference: String,
}

fn has_commit_graph() -> bool {
    let output = Command::new("git")
        .args(["rev-parse", "--git-path", "objects/info"])
        .output();

    let objects_info = match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => return false,
    };

    let objects_info_path = PathBuf::from(&objects_info);

    objects_info_path.join("commit-graph").exists()
        || objects_info_path
            .join("commit-graphs")
            .join("commit-graph-chain")
            .exists()
}

fn get_merge_base(commits: &[(&str, &str)]) -> Option<String> {
    if commits.is_empty() {
        return None;
    }

    let output = Command::new("git")
        .args(
            std::iter::once("merge-base")
                .chain(std::iter::once("--octopus"))
                .chain(commits.iter().map(|(hash, _)| *hash)),
        )
        .output()
        .expect("Failed to execute git merge-base");

    let base = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    if base.is_empty() {
        None
    } else {
        Some(base)
    }
}

fn get_topo_order(reference: &str, exclude_base: Option<&str>) -> Vec<String> {
    let mut args = vec!["rev-list", "--topo-order", reference];
    if let Some(base) = exclude_base {
        args.push(base);
    }

    let output = Command::new("git")
        .args(&args)
        .output()
        .expect("Failed to execute git rev-list");

    if !output.status.success() {
        eprintln!(
            "git rev-list failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        std::process::exit(1);
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .collect()
}

fn main() {
    let args = Args::parse();

    if !has_commit_graph() {
        eprintln!(
            "warning: no commit-graph found. For better performance, run: git commit-graph write"
        );
    }

    // Read input
    let reader: Box<dyn BufRead> = if args.input == "-" {
        Box::new(io::stdin().lock())
    } else {
        Box::new(
            BufReader::new(
                std::fs::File::open(&args.input)
                    .unwrap_or_else(|e| panic!("Failed to open input file '{}': {}", args.input, e)),
            ),
        )
    };

    let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();

    // Parse commits
    let mut commits = match parse_commits(&lines) {
        Ok(c) => c,
        Err(line_num) => {
            eprintln!(
                "error: line {} has leading whitespace, which is not allowed",
                line_num
            );
            std::process::exit(1);
        }
    };

    if commits.is_empty() {
        return;
    }

    // Find merge base to limit traversal range
    let commit_refs: Vec<(&str, &str)> = commits
        .iter()
        .map(|(h, l)| (h.as_str(), l.as_str()))
        .collect();
    let merge_base = get_merge_base(&commit_refs);
    let exclude_base = merge_base.as_ref().map(|b| format!("^{}", b));

    // Get topological order
    let mut topo_order = get_topo_order(
        &args.reference,
        exclude_base.as_deref(),
    );

    // If merge-base is in the input commits, append it to the end
    // (it's the oldest among all input commits)
    if let Some(ref base) = merge_base {
        if commits.iter().any(|(h, _)| h == base) {
            topo_order.push(base.clone());
        }
    }

    let topo_refs: Vec<&str> = topo_order.iter().map(|s| s.as_str()).collect();

    // Check if all commits are in the reference branch
    let missing: Vec<&str> = commits
        .iter()
        .filter(|(hash, _)| !topo_refs.contains(&hash.as_str()))
        .map(|(hash, _)| hash.as_str())
        .collect();

    if !missing.is_empty() {
        eprintln!(
            "error: the following commits are not reachable from '{}':",
            args.reference
        );
        for hash in &missing {
            eprintln!("  {}", hash);
        }
        std::process::exit(1);
    }

    // Sort commits
    sort_by_topo_order(&mut commits, &topo_refs);

    // Write output
    let mut writer: Box<dyn Write> = if args.output == "-" {
        Box::new(io::stdout().lock())
    } else {
        Box::new(
            std::fs::File::create(&args.output)
                .unwrap_or_else(|e| panic!("Failed to create output file '{}': {}", args.output, e)),
        )
    };

    for (_, line) in &commits {
        writeln!(writer, "{}", line).expect("Failed to write output");
    }
}
