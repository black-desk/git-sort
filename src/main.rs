use clap::Parser;
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
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

fn main() {
    let args = Args::parse();

    // Read input
    let reader: Box<dyn BufRead> = if args.input == "-" {
        Box::new(io::stdin().lock())
    } else {
        Box::new(BufReader::new(
            std::fs::File::open(&args.input)
                .unwrap_or_else(|e| panic!("Failed to open input file '{}': {}", args.input, e)),
        ))
    };

    let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();

    // Parse commits: extract hash and keep original line
    let mut commits: Vec<(String, String)> = Vec::new(); // (hash, original_line)
    for line in &lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let hash = line.split('\t').next().unwrap_or(line);
        commits.push((hash.to_string(), line.to_string()));
    }

    if commits.is_empty() {
        return;
    }

    // Find the common ancestor of all input commits to limit traversal range
    let merge_base_output = Command::new("git")
        .args(
            std::iter::once("merge-base")
                .chain(std::iter::once("--octopus"))
                .chain(commits.iter().map(|(hash, _)| hash.as_str())),
        )
        .output()
        .expect("Failed to execute git merge-base");

    let merge_base = String::from_utf8_lossy(&merge_base_output.stdout)
        .trim()
        .to_string();

    // Get topological order from git rev-list
    // Use ^<merge_base> to limit traversal to only the relevant range
    let exclude_base = if !merge_base.is_empty() {
        Some(format!("^{}", merge_base))
    } else {
        None
    };

    let output = Command::new("git")
        .args(["rev-list", "--topo-order", &args.reference])
        .args(exclude_base.as_ref())
        .output()
        .expect("Failed to execute git rev-list");

    if !output.status.success() {
        eprintln!(
            "git rev-list failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        std::process::exit(1);
    }

    let topo_output = String::from_utf8_lossy(&output.stdout);
    let topo_order: Vec<&str> = topo_output.lines().map(|l| l.trim()).collect();

    // Build a map from hash to its position in topological order
    let topo_map: HashMap<&str, usize> = topo_order
        .iter()
        .enumerate()
        .map(|(i, h)| (*h, i))
        .collect();

    // Sort commits by their position in topological order
    // Commits not found in topo order will be placed at the end
    commits.sort_by_key(|(hash, _)| topo_map.get(hash.as_str()).unwrap_or(&usize::MAX));

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
